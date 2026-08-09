//! O controller da própria conta. Não sai do módulo.

use portmaster_app::commands::account::{ChangePasswordCommand, UpdateAccountCommand};
use portmaster_app::context::UserContext;
use portmaster_app::queries::account::GetAccountQuery;
use portmaster_app::services::AccountUseCase;

use crate::controllers::account_controller::AccountController;
use crate::error::api_error::ApiError;
use crate::wire::vo::account::account_password_change_x_request::AccountPasswordChangeXRequest;
use crate::wire::vo::account::account_profile_x_response::AccountProfileXResponse;
use crate::wire::vo::account::account_update_x_request::AccountUpdateXRequest;

/// Os handlers de conta, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct AccountControllerImpl<A> {
    /// O caso de uso de conta.
    account: A,
}

impl<A: AccountUseCase> AccountControllerImpl<A> {
    /// Monta o controller.
    pub(crate) const fn new(account: A) -> Self {
        Self { account }
    }

    /// O perfil pelo lado de leitura, já na forma do fio.
    async fn profile(&self, context: UserContext) -> Result<AccountProfileXResponse, ApiError> {
        let view = self
            .account
            .get(GetAccountQuery { context })
            .await
            .map_err(ApiError::of_app)?;

        Ok(AccountProfileXResponse::of(view))
    }
}

impl<A: AccountUseCase + Clone + Send + Sync + 'static> AccountController
    for AccountControllerImpl<A>
{
    async fn get(&self, context: UserContext) -> Result<AccountProfileXResponse, ApiError> {
        self.profile(context).await
    }

    async fn update(
        &self,
        context: UserContext,
        request: AccountUpdateXRequest,
    ) -> Result<AccountProfileXResponse, ApiError> {
        self.account
            .update(UpdateAccountCommand {
                context: context.clone(),
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        self.profile(context).await
    }

    /// Troca a senha da própria conta.
    ///
    /// A senha atual vai junto e é conferida no `app`: um token roubado não deve
    /// bastar para trocar a senha e expulsar o dono.
    async fn change_password(
        &self,
        context: UserContext,
        request: AccountPasswordChangeXRequest,
    ) -> Result<(), ApiError> {
        self.account
            .change_password(ChangePasswordCommand {
                context,
                current_password: request.current_password.unwrap_or_default(),
                new_password: request.new_password.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)
    }
}
