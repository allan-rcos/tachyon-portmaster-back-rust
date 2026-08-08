//! O perfil do próprio usuário.

use crate::error::api_error::ApiError;
use crate::wire::dto::account::roles_of;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::AccountView;

/// Monta a tabela do perfil.
pub(crate) struct AccountProfileResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: AccountView,
}

impl AccountProfileResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: AccountView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for AccountProfileResponseFactory {
    type Table = fbs::account::AccountProfileResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::account::AccountProfileResponse {
            id: Some(self.source.id.clone()),
            name: Some(self.source.name.clone()),
            email: Some(self.source.email.clone()),
            roles: Some(roles_of(&self.source.roles)?),
        })
    }
}
