//! Os testes de `decode_context`.

use super::*;
use crate::wire::vo::auth::login_x_request::LoginXRequest;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn o_corpo_e_lido_no_formato_do_escopo() {
    let request: LoginXRequest = DecodeContext::scope(MediaType::Json, async {
        DecodeContext.decode(br#"{"email":"ana@portmaster.local","password":"Portmaster1"}"#)
    })
    .await
    .expect("o corpo é JSON válido");

    assert_eq!(request.email.as_deref(), Some("ana@portmaster.local"));
}

/// O mesmo corpo, lido pela strategy errada, é lixo — e vira 400, não
/// pânico.
#[tokio::test]
async fn o_formato_errado_recusa_o_corpo_sem_entrar_em_panico() {
    let error = DecodeContext::scope(MediaType::FlatBuffers, async {
        DecodeContext.decode::<LoginXRequest>(br#"{"email":"ana@portmaster.local"}"#)
    })
    .await
    .expect_err("um JSON não é um buffer");

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
}
