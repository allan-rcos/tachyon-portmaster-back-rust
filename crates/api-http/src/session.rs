//! A sessão da requisição corrente.
//!
//! O contexto extraído do token vive num `task_local` **desta camada** — nem o
//! `app` nem as camadas de baixo o alcançam. Elas recebem o contexto por
//! argumento, dentro do Command, e é o handler que faz a ponte.
//!
//! ## O gate está nos getters, não nos middlewares
//!
//! [`Session::current_user`] falha se o middleware de token não rodou antes.
//! Parece redundante — se o middleware está no router, ele rodou — mas é o que
//! transforma um erro de ordenação da stack em falha imediata e nomeada, em vez
//! de um `None` silencioso que o handler leria como "não há sessão" e
//! responderia 401 para todo mundo.
//!
//! A diferença aparece no dia em que alguém reordena os `.layer()`: com o gate,
//! o primeiro request explica o problema; sem ele, a API simplesmente para de
//! autenticar e ninguém sabe por quê.

use std::future::Future;

use portmaster_app::context::UserContext;
use portmaster_app::{Logger as _, SystemLogger};

use crate::error::api_error::ApiError;

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

/// Acesso à sessão corrente.
pub struct Session;

impl Session {
    /// Executa `future` com o resultado da validação de token.
    ///
    /// Chamado pelo middleware de token, uma vez por requisição. É a única forma
    /// de preencher o armazenamento: um `task_local` não pode ser escrito de
    /// dentro, só envolvido.
    pub(crate) async fn scope<F: Future>(user: Option<UserContext>, future: F) -> F::Output {
        VALIDATED.scope(true, USER.scope(user, future)).await
    }

    /// O usuário da sessão, ou `None` em rota pública.
    ///
    /// Falha — e não devolve `None` — quando o middleware de token não rodou.
    pub(crate) fn current_user() -> Result<Option<UserContext>, ApiError> {
        Self::gate()?;

        Ok(USER.try_with(Clone::clone).ok().flatten())
    }

    /// O usuário da sessão, ou `401`.
    ///
    /// O atalho que todo handler protegido usa. O **401 é o único status que
    /// nasce nesta camada**: falta de sessão é a única coisa que o `app` não tem
    /// como saber.
    pub(crate) fn require_user() -> Result<UserContext, ApiError> {
        Self::current_user()?.ok_or_else(ApiError::unauthenticated)
    }

    /// Confere que houve validação de token nesta requisição.
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
                );

                Err(ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "sessão indisponível",
                ))
            }
        }
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
        Session::scope(Some(context()), async {
            let user = Session::require_user().expect("há sessão");
            assert_eq!(user.id, "u1");
        })
        .await;
    }

    /// O middleware rodou e não achou token — que é diferente de não ter
    /// rodado.
    #[tokio::test]
    async fn rota_publica_tem_escopo_mas_nao_usuario() {
        Session::scope(None, async {
            assert_eq!(Session::current_user().unwrap(), None);
            assert_eq!(
                Session::require_user().err().map(|e| e.status()),
                Some(StatusCode::UNAUTHORIZED)
            );
        })
        .await;
    }

    /// 500 e não 401: o cliente não fez nada errado — a stack do router está
    /// montada fora de ordem, e responder 401 esconderia isso atrás de um
    /// "faça login" que nunca vai funcionar.
    #[tokio::test]
    async fn sem_o_middleware_o_erro_e_nosso_e_nao_do_cliente() {
        assert_eq!(
            Session::current_user().err().map(|e| e.status()),
            Some(StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    #[tokio::test]
    async fn escopos_de_requisicoes_diferentes_nao_se_misturam() {
        let primeira = tokio::spawn(Session::scope(Some(context()), async {
            Session::current_user().unwrap().map(|u| u.id)
        }));

        let segunda = tokio::spawn(Session::scope(None, async {
            Session::current_user().unwrap().map(|u| u.id)
        }));

        let (primeira, segunda) = tokio::join!(primeira, segunda);

        assert_eq!(primeira.unwrap(), Some("u1".to_owned()));
        assert_eq!(segunda.unwrap(), None);
    }
}
