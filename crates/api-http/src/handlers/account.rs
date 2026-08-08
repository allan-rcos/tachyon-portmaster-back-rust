//! `/account` — o que um usuário faz com a própria conta.
//!
//! Nenhuma destas rotas pede permissão: o alvo é sempre quem está agindo, e o id
//! vem do contexto da sessão — nunca do corpo. É o `app` que garante isso; aqui
//! só se repassa o contexto.
//!
//! ## Por que a escrita relê antes de responder
//!
//! `PUT /account` responde com o **perfil**, que carrega quantos usuários cada
//! papel tem. Esse número é consulta, não regra: o objeto de domínio que a
//! escrita devolve não o conhece, e publicá-lo zerado seria informar um valor
//! errado em vez de nenhum. Reler pelo lado de leitura é o que mantém a resposta
//! igual à do `GET` que vem logo depois.

use portmaster_app::commands::account::ChangePasswordCommand;
use portmaster_app::commands::account::UpdateAccountCommand;
use portmaster_app::context::UserContext;
use portmaster_app::queries::account::GetAccountQuery;
use portmaster_app::services::AccountUseCase;

use crate::error::api_error::ApiError;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::account::account_profile_response_factory::AccountProfileResponseFactory;
use crate::wire::dto::account::account_update_request_factory::AccountUpdateRequestFactory;
use crate::wire::dto::account::password_change_request_factory::PasswordChangeRequestFactory;
use crate::wire::no_content::NoContent;
use crate::wire::wire::Wire;

/// Os handlers da conta própria.
pub struct AccountHandlers<A> {
    account: A,
}

impl<A: AccountUseCase> AccountHandlers<A> {
    /// Monta os handlers.
    pub(crate) const fn new(account: A) -> Self {
        Self { account }
    }

    /// `GET /account`
    pub(crate) async fn get(&self, wire: Wire) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        Ok(ApiResponse::ok(wire, self.profile(context).await?))
    }

    /// `PUT /account`
    pub(crate) async fn update(
        &self,
        wire: Wire,
        Body(request): Body<AccountUpdateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        self.account
            .update(UpdateAccountCommand {
                context: context.clone(),
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, self.profile(context).await?))
    }

    /// `PUT /account/password`
    pub(crate) async fn change_password(
        &self,
        Body(request): Body<PasswordChangeRequestFactory>,
    ) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.account
            .change_password(ChangePasswordCommand {
                context,
                current_password: request.current_password.unwrap_or_default(),
                new_password: request.new_password.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        // Sem corpo de propósito: o cookie de sessão continua valendo, e a
        // resposta não tem nada a dizer que o cliente já não saiba.
        Ok(NoContent::new())
    }

    /// O perfil de quem está na sessão, pelo lado de leitura.
    async fn profile(
        &self,
        context: UserContext,
    ) -> Result<AccountProfileResponseFactory, ApiError> {
        let view = self
            .account
            .get(GetAccountQuery { context })
            .await
            .map_err(ApiError::of_app)?;

        Ok(AccountProfileResponseFactory::of(view))
    }
}
