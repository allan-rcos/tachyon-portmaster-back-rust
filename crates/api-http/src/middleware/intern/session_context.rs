//! A sessão da tarefa corrente.
//!
//! > **Lacuna conhecida do `lint-exports`:** os task-locals abaixo nascem de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta três itens na prática, não um.

use std::future::Future;

use portmaster_app::context::UserContext;
use portmaster_app::{Logger as _, SystemLogger};

use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;

tokio::task_local! {
    /// Se o middleware de token já rodou nesta requisição.
    ///
    /// Separado do usuário de propósito: "o token foi conferido e não havia
    /// sessão" e "ninguém conferiu o token ainda" são situações diferentes, e
    /// guardar só o usuário as tornaria indistinguíveis.
    static VALIDATED: bool;

    /// O usuário da requisição, quando há sessão.
    static USER: Option<UserContext>;
}

/// O adaptador que serve a sessão do escopo corrente.
///
/// ZST: a sessão é da tarefa, não do objeto.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SessionContext;

impl SessionContext {
    /// Roda `future` com o resultado da validação de token instalado.
    ///
    /// `pub(super)`: é o escritor, e só o layer irmão o alcança. É a garantia
    /// que o desenho existe para dar — de fora deste módulo não há como
    /// acrescentar um usuário à sessão sem passar pela conferência do token.
    pub(super) async fn scope<F: Future>(user: Option<UserContext>, future: F) -> F::Output {
        VALIDATED.scope(true, USER.scope(user, future)).await
    }

    /// Confirma que o escopo de sessão está instalado.
    ///
    /// Não há caminho que marque `false` hoje: o middleware sempre entra no
    /// escopo com `true`, com ou sem sessão. O braço existe para o dia em que
    /// alguém acrescentar um estado intermediário.
    fn gate() -> Result<(), ApiError> {
        match VALIDATED.try_with(|validated| *validated) {
            Ok(true) => Ok(()),
            Ok(false) => Err(ApiError::unauthenticated()),
            Err(_) => {
                SystemLogger::get().error(
                    "o middleware de token não executou: a ordem dos layers do router está errada",
                    [],
                );

                Err(ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "sessão indisponível",
                ))
            }
        }
    }
}

impl SessionPort for SessionContext {
    fn current_user(&self) -> Result<Option<UserContext>, ApiError> {
        Self::gate()?;

        Ok(USER.try_with(Clone::clone).ok().flatten())
    }

    fn require_user(&self) -> Result<UserContext, ApiError> {
        self.current_user()?.ok_or_else(ApiError::unauthenticated)
    }
}

#[cfg(test)]
#[path = "tests/session_context_test.rs"]
mod tests;
