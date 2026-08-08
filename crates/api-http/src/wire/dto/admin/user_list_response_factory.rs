//! A listagem de usuários.

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::user_admin_response_factory::UserAdminResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::UserListView;

/// Monta a tabela da listagem.
pub(crate) struct UserListResponseFactory {
    source: UserListView,
}

impl UserListResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: UserListView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for UserListResponseFactory {
    type Table = fbs::admin::UserListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::admin::UserListResponse {
            data: Some(
                self.source
                    .items
                    .iter()
                    .map(|item| UserAdminResponseFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        })
    }
}
