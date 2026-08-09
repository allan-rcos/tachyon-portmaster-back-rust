//! A fábrica de consultas do `MariaDB`.

use crate::entity::codec::Codec;
use crate::query::dql::get_account_dql::GetAccountDql;
use crate::query::dql::get_container_dql::GetContainerDql;
use crate::query::dql::get_product_dql::GetProductDql;
use crate::query::dql::get_role_dql::GetRoleDql;
use crate::query::dql::list_container_summaries_dql::ListContainerSummariesDql;
use crate::query::dql::list_containers_dql::ListContainersDql;
use crate::query::dql::list_products_dql::ListProductsDql;
use crate::query::dql::list_roles_dql::ListRolesDql;
use crate::query::dql::list_users_dql::ListUsersDql;
use crate::query::dql::metrics_dql::MetricsDql;
use crate::query::params::{ContainerListParams, ListParams, SummaryListParams, UserListParams};
use crate::query::views::{
    AccountView, ContainerListView, ContainerSummaryListView, ContainerViewItem, MetricsView,
    ProductListView, ProductViewItem, RoleListView, RoleViewItem, UserListView,
};
use crate::query::QueryFactory;
use crate::query::SqlDql;

/// A implementação da fábrica de DQLs.
#[derive(Clone)]
pub(crate) struct MariadbQueryFactory;

impl MariadbQueryFactory {
    /// Monta a fábrica.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl QueryFactory for MariadbQueryFactory {
    fn get_account(
        &self,
        user_id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<AccountView>>> {
        Ok(GetAccountDql::new(Codec::decode_id(user_id)?))
    }

    fn get_container(
        &self,
        id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<ContainerViewItem>>> {
        Ok(GetContainerDql::new(Codec::decode_id(id)?))
    }

    fn get_product(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<ProductViewItem>>> {
        Ok(GetProductDql::new(Codec::decode_id(id)?))
    }

    fn get_role(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<RoleViewItem>>> {
        Ok(GetRoleDql::new(Codec::decode_id(id)?))
    }

    fn list_containers(
        &self,
        params: ContainerListParams,
    ) -> impl SqlDql<View = ContainerListView> {
        ListContainersDql::new(params)
    }

    fn list_container_summaries(
        &self,
        params: SummaryListParams,
    ) -> anyhow::Result<impl SqlDql<View = ContainerSummaryListView>> {
        let id = params.id.as_deref().map(Codec::decode_id).transpose()?;

        Ok(ListContainerSummariesDql::new(params, id))
    }

    fn list_products(&self, params: ListParams) -> impl SqlDql<View = ProductListView> {
        ListProductsDql::new(params)
    }

    fn list_roles(&self, params: ListParams) -> impl SqlDql<View = RoleListView> {
        ListRolesDql::new(params)
    }

    fn list_users(&self, params: UserListParams) -> impl SqlDql<View = UserListView> {
        ListUsersDql::new(params)
    }

    fn metrics(&self) -> impl SqlDql<View = MetricsView> {
        MetricsDql::new()
    }
}
