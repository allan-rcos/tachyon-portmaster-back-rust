//! Embarcar e desembarcar carga.
//!
//! Os dois seguem exatamente o mesmo caminho e diferem só no método do
//! TableModule. O que o `domain` devolve não é uma carga: é uma **mudança**
//! (`ManifestChange`) descrevendo o contêiner no estado novo, a linha de carga
//! resultante e o evento a registrar. Persistir essa mudança é o trabalho desta
//! camada, e é um só para os dois casos.

use portmaster_domain::container::Container;
use portmaster_domain::manifest::{ManifestChange, ManifestTM};
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::repository::{ContainerRepository, ManifestRepository, ProductRepository};

use crate::authorization::{slug, RequiresPermission};
use crate::cache;
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Movimentar carga — embarque ou desembarque.
///
/// Um comando só para os dois: o que muda é a operação pedida, não os dados.
#[derive(Debug, Clone)]
pub struct MoveItemCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do contêiner, em base62.
    pub container_id: String,
    /// Id do produto, em base62.
    pub product_id: String,
    /// Quantidade a movimentar.
    pub quantity: f64,
}

/// O que a apresentação pode pedir sobre manifesto.
///
/// Os dois devolvem o **contêiner** no estado novo, e não a linha movimentada:
/// quem embarca quer saber quanto o contêiner passou a pesar e se saiu de vazio,
/// que é o que decide o próximo movimento. A linha em si o chamador já tem — foi
/// ele quem a pediu.
#[trait_variant::make(Send)]
pub trait ManifestUseCase {
    /// Embarca carga num contêiner, e devolve o contêiner resultante.
    async fn load(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError>;

    /// Desembarca carga de um contêiner, e devolve o contêiner resultante.
    async fn unload(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct ManifestUseCaseImpl<CR, PR, MR, T, C, U> {
    containers: CR,
    products: PR,
    manifest: MR,
    manifest_tm: T,
    cache: C,
    unit_of_work: U,
    load_permission: RequiresPermission,
    unload_permission: RequiresPermission,
}

impl<CR, PR, MR, T, C, U> ManifestUseCaseImpl<CR, PR, MR, T, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) fn new(
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
            load_permission: RequiresPermission::new(slug::MANIFEST_LOAD),
            unload_permission: RequiresPermission::new(slug::MANIFEST_UNLOAD),
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
    /// Carrega os três envolvidos, pede a mudança ao TableModule e a persiste.
    /// O `apply` é o único ponto de diferença entre os dois casos.
    async fn r#move(
        &self,
        command: MoveItemCommand,
        apply: impl Fn(
            &T,
            &dyn Container,
            &dyn portmaster_domain::product::Product,
            f64,
            Option<&dyn portmaster_domain::manifest::ManifestCargo>,
        ) -> Result<Box<dyn ManifestChange>, AppError>,
    ) -> Result<Box<dyn Container>, AppError> {
        let container = transaction(&self.unit_of_work, async {
            let container = self
                .containers
                .find_by_id(&command.container_id)
                .await?
                .ok_or_else(|| AppError::not_found("contêiner", &command.container_id))?;

            let product = self
                .products
                .find_by_id(&command.product_id)
                .await?
                .ok_or_else(|| AppError::not_found("produto", &command.product_id))?;

            // A carga atual pode não existir — é o primeiro embarque daquele
            // produto. Ausência aqui é dado, não falha.
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

            // Consumir a mudança depois de gravá-la: o contêiner que sai daqui é
            // o mesmo que foi persistido, sem uma segunda leitura que poderia
            // divergir.
            Ok(change.into_container())
        })
        .await?;

        cache::invalidate(&self.cache, cache::MANIFEST_WRITE).await?;

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
