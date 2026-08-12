//! Os testes de `request_id_context`.

use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn dentro_do_escopo_o_id_esta_disponivel() {
    RequestIdContext::scope("abc123".to_owned(), async {
        assert_eq!(RequestIdContext.current(), Some("abc123".to_owned()));
    })
    .await;
}

/// Faltar id não é erro: correlação ausente não impede ninguém de responder.
#[tokio::test]
async fn fora_do_escopo_nao_ha_id() {
    assert_eq!(RequestIdContext.current(), None);
}

#[tokio::test]
async fn escopos_de_requisicoes_diferentes_nao_se_misturam() {
    let first = tokio::spawn(RequestIdContext::scope("um".to_owned(), async {
        RequestIdContext.current()
    }));
    let second = tokio::spawn(RequestIdContext::scope("dois".to_owned(), async {
        RequestIdContext.current()
    }));

    let (first, second) = tokio::join!(first, second);

    assert_eq!(
        first.expect("a tarefa não entra em pânico"),
        Some("um".to_owned())
    );
    assert_eq!(
        second.expect("a tarefa não entra em pânico"),
        Some("dois".to_owned())
    );
}
