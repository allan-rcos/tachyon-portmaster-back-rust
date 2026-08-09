//! A orquestração de carga e telemetria.

use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::manifest::MoveItemCommand;
use crate::error::AppError;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::ManifestUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::Container;
use portmaster_domain::models::ManifestChange;
use portmaster_domain::table_modules::ManifestTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::repository::{ContainerRepository, ManifestRepository, ProductRepository};

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct ManifestUseCaseImpl<CR, PR, MR, T, C, U> {
    /// Persistência de contêineres.
    containers: CR,
    /// Persistência de produtos.
    products: PR,
    /// Persistência do manifesto e da telemetria.
    manifest: MR,
    /// As regras de embarque e desembarque.
    manifest_tm: T,
    /// O cache de leitura, para o read-through e a invalidação.
    cache: C,
    /// Quem abre e fecha a transação.
    unit_of_work: U,
    /// A permissão exigida para load.
    load_permission: RequiresPermission,
    /// A permissão exigida para unload.
    unload_permission: RequiresPermission,
}

impl<CR, PR, MR, T, C, U> ManifestUseCaseImpl<CR, PR, MR, T, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) const fn new(
        containers: CR,
        products: PR,
        manifest: MR,
        manifest_tm: T,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            containers,
            products,
            manifest,
            manifest_tm,
            cache,
            unit_of_work,
            load_permission: RequiresPermission::new(PermissionSlug::MANIFEST_LOAD),
            unload_permission: RequiresPermission::new(PermissionSlug::MANIFEST_UNLOAD),
        }
    }
}

impl<CR, PR, MR, T, C, U> ManifestUseCaseImpl<CR, PR, MR, T, C, U>
where
    CR: ContainerRepository + Send + Sync,
    PR: ProductRepository + Send + Sync,
    MR: ManifestRepository + Send + Sync,
    T: ManifestTM + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// A moldura de embarque e desembarque.
    ///
    /// Carrega os três envolvidos, pede a mudança ao `TableModule` e a persiste.
    /// O `apply` é o único ponto de diferença entre os dois casos.
    /// Embarca ou desembarca, conforme a operação que `apply` conduz.
    ///
    /// A carga atual pode não existir — é o primeiro embarque daquele produto.
    /// Ausência aqui é **dado, não falha**.
    ///
    /// A mudança é consumida depois de gravada: o contêiner que sai daqui é o
    /// mesmo que foi persistido, sem uma segunda leitura que poderia divergir.
    async fn r#move(
        &self,
        command: MoveItemCommand,
        apply: impl Fn(
            &T,
            &dyn Container,
            &dyn portmaster_domain::models::Product,
            f64,
            Option<&dyn portmaster_domain::models::ManifestCargo>,
        ) -> Result<Box<dyn ManifestChange>, AppError>,
    ) -> Result<Box<dyn Container>, AppError> {
        let container = Transaction::run(&self.unit_of_work, async {
            let container = self
                .containers
                .find_by_id(&command.container_id)
                .await?
                .ok_or_else(|| AppError::missing("contêiner", &command.container_id))?;

            let product = self
                .products
                .find_by_id(&command.product_id)
                .await?
                .ok_or_else(|| AppError::missing("produto", &command.product_id))?;

            let current = self
                .manifest
                .find_cargo(&command.container_id, &command.product_id)
                .await?;

            let change = apply(
                &self.manifest_tm,
                container.as_ref(),
                product.as_ref(),
                command.quantity,
                current.as_deref(),
            )?;

            self.persist(change.as_ref()).await?;

            Ok(change.into_container())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::MANIFEST_WRITE).await?;

        Ok(container)
    }

    /// Grava a mudança que o `domain` descreveu.
    ///
    /// Os três ramos são exclusivos e vêm do próprio `ManifestChange`: limpar o
    /// manifesto inteiro (o desembarque zerou o contêiner), apagar a linha (o
    /// produto saiu por completo) ou gravá-la (embarque, ou desembarque
    /// parcial). Decidir aqui qual dos três seria reimplementar a regra do
    /// domínio numa segunda cópia.
    async fn persist(&self, change: &dyn ManifestChange) -> Result<(), AppError> {
        let container_id = change.container().id().to_owned();

        self.containers.update(change.container()).await?;

        if change.clear_manifest() {
            self.manifest.clear_manifest(&container_id).await?;
        } else if let Some(cargo) = change.cargo() {
            self.manifest.upsert_cargo(cargo).await?;
        } else {
            self.manifest
                .delete_cargo(&container_id, change.product_id())
                .await?;
        }

        self.manifest
            .insert_telemetry(
                &container_id,
                change.event(),
                Some(&format!("Product {}", change.product_id())),
            )
            .await?;

        Ok(())
    }
}

impl<CR, PR, MR, T, C, U> ManifestUseCase for ManifestUseCaseImpl<CR, PR, MR, T, C, U>
where
    CR: ContainerRepository + Send + Sync,
    PR: ProductRepository + Send + Sync,
    MR: ManifestRepository + Send + Sync,
    T: ManifestTM + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn load(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError> {
        self.load_permission.authorize(&command.context)?;

        self.r#move(command, |tm, container, product, quantity, current| {
            Ok(tm.load(container, product, quantity, current)?)
        })
        .await
    }

    async fn unload(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError> {
        self.unload_permission.authorize(&command.context)?;

        self.r#move(command, |tm, container, product, quantity, current| {
            Ok(tm.unload(container, product, quantity, current)?)
        })
        .await
    }
}
