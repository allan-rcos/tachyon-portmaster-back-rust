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

use portmaster_app::account::{
    AccountUseCase, ChangePasswordCommand, GetAccountQuery, UpdateAccountCommand,
};
use portmaster_app::context::UserContext;

use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, Negotiated, NoContent};
use crate::wire::tables as fbs;

/// Os handlers da conta própria.
pub(crate) struct AccountHandlers<A> {
    account: A,
}

impl<A: AccountUseCase> AccountHandlers<A> {
    /// Monta os handlers.
    pub(crate) fn new(account: A) -> Self {
        Self { account }
    }

    /// `GET /account`
    pub(crate) async fn get(
        &self,
        accept: Accept,
    ) -> Result<Negotiated<fbs::account::AccountProfileResponse>, ApiError> {
        let context = Session::require_user()?;

        Ok(Negotiated::ok(accept, self.profile(context).await?))
    }

    /// `PUT /account`
    pub(crate) async fn update(
        &self,
        accept: Accept,
        Body(request): Body<fbs::account::AccountUpdateRequest>,
    ) -> Result<Negotiated<fbs::account::AccountProfileResponse>, ApiError> {
        let context = Session::require_user()?;

        self.account
            .update(UpdateAccountCommand {
                context: context.clone(),
                name: request.name,
                email: request.email,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, self.profile(context).await?))
    }

    /// `PUT /account/password`
    pub(crate) async fn change_password(
        &self,
        Body(request): Body<fbs::account::AccountPasswordChangeRequest>,
    ) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.account
            .change_password(ChangePasswordCommand {
                context,
                current_password: request.current_password,
                new_password: request.new_password,
            })
            .await
            .map_err(app_error_to_status)?;

        // Sem corpo de propósito: o cookie de sessão continua valendo, e a
        // resposta não tem nada a dizer que o cliente já não saiba.
        Ok(NoContent::new())
    }

    /// O perfil de quem está na sessão, pelo lado de leitura.
    async fn profile(
        &self,
        context: UserContext,
    ) -> Result<fbs::account::AccountProfileResponse, ApiError> {
        let view = self
            .account
            .get(GetAccountQuery { context })
            .await
            .map_err(app_error_to_status)?;

        Ok(view.into())
    }
}
