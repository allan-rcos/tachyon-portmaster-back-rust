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
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use portmaster_app::context::RoleContext;
    use pretty_assertions::assert_eq;

    fn context() -> UserContext {
        UserContext {
            id: "u1".into(),
            name: "Ana".into(),
            email: "ana@portmaster.local".into(),
            roles: vec![RoleContext {
                id: "r1".into(),
                name: "Operador".into(),
                permissions: vec!["container:seal".into()],
            }],
        }
    }

    #[tokio::test]
    async fn dentro_do_escopo_a_sessao_esta_disponivel() {
        SessionContext::scope(Some(context()), async {
            let user = SessionContext.require_user().expect("há sessão");
            assert_eq!(user.id, "u1");
        })
        .await;
    }

    /// O middleware rodou e não achou token — que é diferente de não ter
    /// rodado.
    #[tokio::test]
    async fn rota_publica_tem_escopo_mas_nao_usuario() {
        SessionContext::scope(None, async {
            assert_eq!(SessionContext.current_user().unwrap(), None);
            assert_eq!(
                SessionContext.require_user().err().map(|e| e.status()),
                Some(StatusCode::UNAUTHORIZED)
            );
        })
        .await;
    }

    /// 500 e não 401: o cliente não fez nada errado — a pilha do router está
    /// montada fora de ordem, e responder 401 esconderia isso atrás de um
    /// "faça login" que nunca vai funcionar.
    #[tokio::test]
    async fn sem_o_middleware_o_erro_e_nosso_e_nao_do_cliente() {
        assert_eq!(
            SessionContext.current_user().err().map(|e| e.status()),
            Some(StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    #[tokio::test]
    async fn escopos_de_requisicoes_diferentes_nao_se_misturam() {
        let primeira = tokio::spawn(SessionContext::scope(Some(context()), async {
            SessionContext.current_user().unwrap().map(|u| u.id)
        }));

        let segunda = tokio::spawn(SessionContext::scope(None, async {
            SessionContext.current_user().unwrap().map(|u| u.id)
        }));

        let (primeira, segunda) = tokio::join!(primeira, segunda);

        assert_eq!(primeira.unwrap(), Some("u1".to_owned()));
        assert_eq!(segunda.unwrap(), None);
    }
}
