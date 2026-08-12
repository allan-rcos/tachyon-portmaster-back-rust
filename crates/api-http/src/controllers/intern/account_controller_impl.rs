//! O controller da própria conta. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::account::{ChangePasswordCommand, UpdateAccountCommand};
use portmaster_app::context::UserContext;
use portmaster_app::error::AccountError;
use portmaster_app::queries::account::GetAccountQuery;
use portmaster_app::services::AccountService;

use crate::controllers::account_controller::AccountController;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::account::account_password_change_x_request::AccountPasswordChangeXRequest;
use crate::wire::vo::account::account_profile_x_response::AccountProfileXResponse;
use crate::wire::vo::account::account_update_x_request::AccountUpdateXRequest;

/// Os handlers de conta, genéricos sobre o service.
#[derive(Clone)]
pub(crate) struct AccountControllerImpl<A, S> {
    /// O service de conta.
    account: A,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<A: AccountService, S: SessionPort> AccountControllerImpl<A, S> {
    /// Monta o controller.
    pub(crate) const fn new(account: A, session: S) -> Self {
        Self { account, session }
    }

    /// O perfil pelo lado de leitura, já na forma do fio.
    async fn profile(&self, context: UserContext) -> Result<AccountProfileXResponse, ApiError> {
        let view = self
            .account
            .get(GetAccountQuery { context })
            .await
            .map_err(to_api)?;

        Ok(AccountProfileXResponse::of(view))
    }
}

impl<A: AccountService + Clone + Send + Sync + 'static, S: SessionPort> AccountController
    for AccountControllerImpl<A, S>
{
    async fn get(self) -> ApiResponse<AccountProfileXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                self.profile(context).await
            }
            .await,
        )
    }

    async fn update(
        self,
        Body(request): Body<AccountUpdateXRequest>,
    ) -> ApiResponse<AccountProfileXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                self.account
                    .update(UpdateAccountCommand {
                        context: context.clone(),
                        name: request.name.unwrap_or_default(),
                        email: request.email.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)?;

                self.profile(context).await
            }
            .await,
        )
    }

    /// Troca a senha da própria conta.
    ///
    /// A senha atual vai junto e é conferida no `app`: um token roubado não deve
    /// bastar para trocar a senha e expulsar o dono.
    async fn change_password(
        self,
        Body(request): Body<AccountPasswordChangeXRequest>,
    ) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.account
                    .change_password(ChangePasswordCommand {
                        context,
                        current_password: request.current_password.unwrap_or_default(),
                        new_password: request.new_password.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
    }
}

/// Traduz a recusa do serviço de conta no status que o cliente recebe.
///
/// A sessão que não descreve mais ninguém e a senha atual que não confere são a
/// mesma coisa aqui: 401. O resto é a recusa comum a toda a camada.
fn to_api(error: AccountError) -> ApiError {
    match error {
        AccountError::InvalidCredentials => {
            ApiError::new(StatusCode::UNAUTHORIZED, error.to_string())
        }
        AccountError::App(shared) => ApiError::of_app(shared),
    }
}
