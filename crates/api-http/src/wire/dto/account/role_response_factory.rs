//! Um papel, como ele aparece dentro de um usuário.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::RoleViewItem;

/// Monta a tabela de um papel.
pub(crate) struct RoleResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: RoleViewItem,
}

impl RoleResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: RoleViewItem) -> Self {
        Self { source }
    }
}

impl ResponseFactory for RoleResponseFactory {
    type Table = fbs::account::RoleResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::account::RoleResponse {
            id: Some(self.source.id.clone()),
            name: Some(self.source.name.clone()),
            permissions: Some(self.source.permissions.clone()),
            user_count: Convert::count(self.source.user_count),
        })
    }
}
