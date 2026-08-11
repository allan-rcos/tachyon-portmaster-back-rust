//! O contrato do controller da própria conta.

use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::account::account_password_change_x_request::AccountPasswordChangeXRequest;
use crate::wire::vo::account::account_profile_x_response::AccountProfileXResponse;
use crate::wire::vo::account::account_update_x_request::AccountUpdateXRequest;

/// Os handlers da conta de quem está na sessão.
#[trait_variant::make(Send)]
pub(crate) trait AccountController: Clone + Sync + 'static {
    /// `GET /account`
    async fn get(self) -> ApiResponse<AccountProfileXResponse>;

    /// `PUT /account`
    async fn update(
        self,
        request: Body<AccountUpdateXRequest>,
    ) -> ApiResponse<AccountProfileXResponse>;

    /// `PUT /account/password`
    async fn change_password(self, request: Body<AccountPasswordChangeXRequest>) -> ApiResponse;
}
