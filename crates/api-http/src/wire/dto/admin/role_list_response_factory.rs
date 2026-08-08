//! A listagem de papéis.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::dto::account::role_response_factory::RoleResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::RoleListView;

/// Monta a tabela da listagem.
pub(crate) struct RoleListResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: RoleListView,
}

impl RoleListResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: RoleListView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for RoleListResponseFactory {
    type Table = fbs::admin::RoleListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::admin::RoleListResponse {
            next_cursor: self.source.next_cursor.clone(),
            data: Some(
                self.source
                    .items
                    .iter()
                    .map(|item| RoleResponseFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            total: Convert::count(self.source.total),
        })
    }
}
