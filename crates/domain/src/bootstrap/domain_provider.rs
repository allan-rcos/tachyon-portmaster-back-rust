//! A borda do `domain`: tudo que sai deste crate sai por aqui.

use crate::config::DomainSecrets;
use crate::id::IdProvider;
use crate::id::{RandomIdGenerator, SequentialIdGenerator};
use crate::table_modules::TableModulesProvider;
use crate::table_modules::{
    AuthTM, ContainerTM, ManifestTM, MarkerGroupTM, MarkerTM, PermissionTM, ProductTM, RoleTM,
    UserTM,
};

/// Os factories do `domain`.
///
/// O que ele serve, o `app` recebe como **contrato**: nenhum tipo concreto do
/// domínio atravessa esta fronteira. Ver
/// `docs/adr/0011-static-providers-one-per-directory.md`.
///
/// O gerador de **identidade de entidade** não aparece aqui, e é o único que
/// não aparece: servi-lo daria a quem está de fora o poder de nomear uma linha
/// sem passar pelo `TableModule` que a valida. Os outros dois emitem id que
/// nunca vira chave, e por isso atravessam.
pub struct DomainProvider;

impl DomainProvider {
    /// Instala a identidade de deploy desta instância.
    ///
    /// É a única configuração que o `domain` tem, e por isso o único `install`
    /// daqui. Instalar de novo troca; o lugar normal de chamar isto é o boot.
    pub fn install_identity(secrets: DomainSecrets) {
        IdProvider::install(secrets);
    }

    /// O gerador de id ordenável, para o `request_id`.
    pub fn sequential_id_generator() -> impl SequentialIdGenerator + use<> {
        IdProvider::sequential()
    }

    /// O gerador de id opaco, para o refresh token.
    pub fn random_id_generator() -> impl RandomIdGenerator + use<> {
        IdProvider::random()
    }

    /// As regras de usuário.
    pub fn user_table_module() -> impl UserTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::user()
    }

    /// As regras de papel.
    pub fn role_table_module() -> impl RoleTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::role()
    }

    /// As regras de produto.
    pub fn product_table_module() -> impl ProductTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::product()
    }

    /// As regras de contêiner.
    pub fn container_table_module() -> impl ContainerTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::container()
    }

    /// As regras de manifesto.
    pub fn manifest_table_module() -> impl ManifestTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::manifest()
    }

    /// As regras de autenticação.
    pub fn auth_table_module() -> impl AuthTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::auth()
    }

    /// As regras de permissão.
    pub fn permission_table_module() -> impl PermissionTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::permission()
    }

    /// As regras de grupo de marcador.
    pub fn marker_group_table_module() -> impl MarkerGroupTM + Send + Sync + Clone + use<> + 'static
    {
        TableModulesProvider::marker_group()
    }

    /// As regras de marcador.
    pub fn marker_table_module() -> impl MarkerTM + Send + Sync + Clone + use<> + 'static {
        TableModulesProvider::marker()
    }
}
