//! Os factories do `domain`.

use crate::id::{RandomIdGenerator, SequentialIdGenerator};
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
///
/// O gerador de **identidade de entidade** não aparece aqui, e é o único que
/// não aparece: servi-lo daria a quem está de fora o poder de nomear uma linha
/// sem passar pelo `TableModule` que a valida. Os outros dois emitem id que
/// nunca vira chave, e por isso atravessam.
pub trait DomainProvider {
    /// Gerador de id ordenável — o `request_id` da apresentação.
    fn sequential_id_generator(&self) -> impl SequentialIdGenerator + use<Self>;

    /// Gerador de id opaco — o refresh token da apresentação.
    fn random_id_generator(&self) -> impl RandomIdGenerator + use<Self>;

    /// `TableModule` de usuário.
    fn user_table_module(&self) -> impl UserTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de papel.
    fn role_table_module(&self) -> impl RoleTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de produto.
    fn product_table_module(&self) -> impl ProductTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de contêiner.
    fn container_table_module(
        &self,
    ) -> impl ContainerTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de manifesto.
    fn manifest_table_module(&self) -> impl ManifestTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de autenticação.
    fn auth_table_module(&self) -> impl AuthTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de permissão.
    fn permission_table_module(
        &self,
    ) -> impl PermissionTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de grupo de marcador.
    fn marker_group_table_module(
        &self,
    ) -> impl MarkerGroupTM + Send + Sync + Clone + use<Self> + 'static;

    /// `TableModule` de marcador.
    fn marker_table_module(&self) -> impl MarkerTM + Send + Sync + Clone + use<Self> + 'static;
}
