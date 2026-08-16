//! A orquestração de carga e telemetria.
//!
//! ## As permissões são privadas
//!
//! Os slugs abaixo são **contrato**: já existem em papéis gravados no banco de
//! quem roda a versão PHP, e renomear qualquer um revoga silenciosamente o
//! acesso de quem o tinha. São `const` privadas porque uma permissão pertence a
//! exatamente um caso de uso — é ele quem a compara com o `UserContext`, e não
//! há segundo lugar no sistema que precise vê-la. O boot as registra chamando
//! `declare_permissions`, sem nunca lê-las.

use portmaster_domain::domain::{Container, ManifestCargo, ManifestChange, Product};
use portmaster_domain::table_modules::ManifestTM;
use portmaster_infra::repository::{
    ContainerRepository, ManifestRepository, ProductRepository, ViewCacheRepository,
};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::manifest::MoveItemCommand;
use crate::commands::metadata::RegisterPermissionCommand;
use crate::error::{AppError, ManifestError};
use crate::services::ManifestService;
use crate::services::MetadataService;

/// Embarcar carga.
const LOAD: &str = "manifest:load";
/// Desembarcar carga.
const UNLOAD: &str = "manifest:unload";

/// O prefixo que um embarque ou desembarque derruba.
///
/// É o do contêiner, não o deste serviço: quem muda é o peso, o status e o
/// resumo de carga, e todos são leitura de contêiner. Este serviço não tem
/// leitura própria a invalidar.
const CONTAINER_CACHE_GROUP: &str = "container";

/// Monta o caso de uso de manifesto.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn manifest_service<CR, PR, MR, T, C>(
    containers: CR,
    products: PR,
    manifest: MR,
    manifest_tm: T,
    views: C,
) -> impl ManifestService + Sync + Clone + use<CR, PR, MR, T, C> + 'static
where
    CR: ContainerRepository + Send + Sync + Clone + 'static,
    PR: ProductRepository + Send + Sync + Clone + 'static,
    MR: ManifestRepository + Send + Sync + Clone + 'static,
    T: ManifestTM + Send + Sync + Clone + 'static,
    C: ViewCacheRepository + Send + Sync + Clone + 'static,
{
    ManifestServiceImpl {
        containers,
        products,
        manifest,
        manifest_tm,
        views,
    }
}

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct ManifestServiceImpl<CR, PR, MR, T, C> {
    /// Persistência de contêineres.
    containers: CR,
    /// Persistência de produtos.
    products: PR,
    /// Persistência do manifesto e da telemetria.
    manifest: MR,
    /// As regras de embarque e desembarque.
    manifest_tm: T,
    /// O cache do lado de leitura.
    views: C,
}

impl<CR, PR, MR, T, C> ManifestServiceImpl<CR, PR, MR, T, C>
where
    CR: ContainerRepository + Send + Sync,
    PR: ProductRepository + Send + Sync,
    MR: ManifestRepository + Send + Sync,
    T: ManifestTM + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    /// A moldura de embarque e desembarque.
    ///
    /// Carrega os três envolvidos, pede a mudança ao `TableModule` e a persiste.
    /// O `apply` é o único ponto de diferença entre os dois casos.
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
            &dyn Product,
            f64,
            Option<&dyn ManifestCargo>,
        ) -> Result<Box<dyn ManifestChange>, ManifestError>,
    ) -> Result<Box<dyn Container>, ManifestError> {
        let container = MasterScope::run(|uow| async move {
            let Some(container) = self.containers.find_by_id(&command.container_id).await? else {
                return Err(ManifestError::MissingContainer(command.container_id));
            };

            let Some(product) = self.products.find_by_id(&command.product_id).await? else {
                return Err(ManifestError::MissingProduct(command.product_id));
            };

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

            uow.commit().await?;

            Ok(change.into_container())
        })
        .await?;

        self.views.invalidate(CONTAINER_CACHE_GROUP).await?;

        Ok(container)
    }

    /// Grava a mudança que o `domain` descreveu.
    ///
    /// Os três ramos são exclusivos e vêm do próprio `ManifestChange`: limpar o
    /// manifesto inteiro (o desembarque zerou o contêiner), apagar a linha (o
    /// produto saiu por completo) ou gravá-la (embarque, ou desembarque
    /// parcial). Decidir aqui qual dos três seria reimplementar a regra do
    /// domínio numa segunda cópia.
    async fn persist(&self, change: &dyn ManifestChange) -> Result<(), ManifestError> {
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

impl<CR, PR, MR, T, C> ManifestService for ManifestServiceImpl<CR, PR, MR, T, C>
where
    CR: ContainerRepository + Send + Sync,
    PR: ProductRepository + Send + Sync,
    MR: ManifestRepository + Send + Sync,
    T: ManifestTM + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ManifestError> {
        for slug in [LOAD, UNLOAD] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    async fn load(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, ManifestError> {
        if !command.context.has_permission(LOAD) {
            return Err(AppError::permission_denied(LOAD).into());
        }

        self.r#move(command, |tm, container, product, quantity, current| {
            Ok(tm.load(container, product, quantity, current)?)
        })
        .await
    }

    async fn unload(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, ManifestError> {
        if !command.context.has_permission(UNLOAD) {
            return Err(AppError::permission_denied(UNLOAD).into());
        }

        self.r#move(command, |tm, container, product, quantity, current| {
            Ok(tm.unload(container, product, quantity, current)?)
        })
        .await
    }
}

#[cfg(test)]
#[path = "tests/manifest_service_impl_test.rs"]
mod tests;
