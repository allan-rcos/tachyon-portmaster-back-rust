//! O contrato de quem monta as consultas de leitura.

use crate::query::params::{ContainerListParams, ListParams, SummaryListParams, UserListParams};
use crate::query::views::{
    AccountView, ContainerListView, ContainerSummaryListView, ContainerViewItem, MetricsView,
    ProductListView, ProductViewItem, RoleListView, RoleViewItem, UserListView,
};
use crate::query::SqlDql;

/// Constrói os descritores de consulta.
///
/// O `app` pede a consulta que quer e recebe algo que só sabe ser executado. Não
/// alcança o SQL, não alcança o cursor, não consegue inventar uma consulta que
/// esta camada não tenha declarado.
///
/// Os métodos por id são falíveis porque um id em base62 pode simplesmente não
/// ser base62 — uma URL inventada. Recusar ali é melhor do que abrir transação e
/// consultar por um número arbitrário.
pub trait QueryFactory {
    /// Um usuário com os papéis dele.
    fn get_account(&self, user_id: &str)
        -> anyhow::Result<impl SqlDql<View = Option<AccountView>>>;

    /// Um contêiner.
    fn get_container(
        &self,
        id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<ContainerViewItem>>>;

    /// Um produto.
    fn get_product(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<ProductViewItem>>>;

    /// Um papel.
    fn get_role(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<RoleViewItem>>>;

    /// A listagem de contêineres.
    fn list_containers(&self, params: ContainerListParams)
        -> impl SqlDql<View = ContainerListView>;

    /// A listagem de contêineres com carga e telemetria recente.
    fn list_container_summaries(
        &self,
        params: SummaryListParams,
    ) -> anyhow::Result<impl SqlDql<View = ContainerSummaryListView>>;

    /// A listagem de produtos.
    fn list_products(&self, params: ListParams) -> impl SqlDql<View = ProductListView>;

    /// A listagem de papéis.
    fn list_roles(&self, params: ListParams) -> impl SqlDql<View = RoleListView>;

    /// A listagem de usuários.
    fn list_users(&self, params: UserListParams) -> impl SqlDql<View = UserListView>;

    /// O painel do pátio.
    fn metrics(&self) -> impl SqlDql<View = MetricsView>;
}
