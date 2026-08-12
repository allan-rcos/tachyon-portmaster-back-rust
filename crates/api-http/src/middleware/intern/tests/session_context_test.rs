//! Os testes de `session_context`.

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
