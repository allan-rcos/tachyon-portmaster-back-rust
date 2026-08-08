//! Um usuário, como um administrador o vê.

use crate::error::api_error::ApiError;
use crate::wire::dto::account::roles_of;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::AccountView;

/// Monta a tabela de um usuário.
pub(crate) struct UserAdminResponseFactory {
    source: AccountView,
}

impl UserAdminResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: AccountView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for UserAdminResponseFactory {
    type Table = fbs::admin::UserAdminResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::admin::UserAdminResponse {
            id: Some(self.source.id.clone()),
            name: Some(self.source.name.clone()),
            email: Some(self.source.email.clone()),
            roles: Some(roles_of(&self.source.roles)?),
        })
    }
}
