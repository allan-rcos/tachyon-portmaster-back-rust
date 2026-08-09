//! Os factories do `domain`.

use crate::table_modules::{
    AuthTM, ContainerTM, ManifestTM, MarkerGroupTM, MarkerTM, PermissionTM, ProductTM, RoleTM,
    UserTM,
};

/// Os factories do `domain`.
///
/// Cada método devolve `impl Trait`: o consumidor recebe o **contrato**, nunca o
/// tipo concreto. O despacho é estático — o compilador monomorfiza o grafo
/// inteiro, então um serviço que não pode ser construído é erro de compilação,
/// não surpresa em produção.
pub trait DomainProvider {
    /// `TableModule` de usuário.
    fn user_table_module(&self) -> impl UserTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de papel.
    fn role_table_module(&self) -> impl RoleTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de produto.
    fn product_table_module(&self) -> impl ProductTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de contêiner.
    fn container_table_module(&self) -> impl ContainerTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de manifesto.
    fn manifest_table_module(&self) -> impl ManifestTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de autenticação.
    fn auth_table_module(&self) -> impl AuthTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de permissão.
    fn permission_table_module(&self) -> impl PermissionTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de grupo de marcador.
    fn marker_group_table_module(&self) -> impl MarkerGroupTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de marcador.
    fn marker_table_module(&self) -> impl MarkerTM + Send + Sync + Clone + use<Self> + 'static;
}
