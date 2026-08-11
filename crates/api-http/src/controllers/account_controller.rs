//! O contrato do controller da própria conta.

use portmaster_app::context::UserContext;

use crate::ports::error::api_error::ApiError;
use crate::wire::vo::account::account_password_change_x_request::AccountPasswordChangeXRequest;
use crate::wire::vo::account::account_profile_x_response::AccountProfileXResponse;
use crate::wire::vo::account::account_update_x_request::AccountUpdateXRequest;

/// Os handlers da conta de quem está na sessão.
#[trait_variant::make(Send)]
pub(crate) trait AccountController: Clone + Sync + 'static {
    /// `GET /account`
    async fn get(&self, context: UserContext) -> Result<AccountProfileXResponse, ApiError>;

    /// `PUT /account`
    async fn update(
        &self,
        context: UserContext,
        request: AccountUpdateXRequest,
    ) -> Result<AccountProfileXResponse, ApiError>;

    /// `PUT /account/password`
    async fn change_password(
        &self,
        context: UserContext,
        request: AccountPasswordChangeXRequest,
    ) -> Result<(), ApiError>;
}
