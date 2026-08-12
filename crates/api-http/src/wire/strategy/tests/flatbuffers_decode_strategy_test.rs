//! Os testes de `flatbuffers_decode_strategy`.

use super::*;
use crate::wire::tables as fbs;
use crate::wire::vo::auth::login_x_request::LoginXRequest;
use pretty_assertions::assert_eq;

#[test]
fn o_corpo_binario_volta_a_ser_a_mesma_mensagem() {
    let sent = fbs::auth::LoginRequest {
        email: "ana@portmaster.local".into(),
        password: "Portmaster1".into(),
    };
    let mut builder = planus::Builder::new();
    let bytes = builder.finish(&sent, None).to_vec();

    let request: LoginXRequest = FlatBuffersDecodeStrategy
        .decode(&bytes)
        .expect("o buffer foi escrito por nós");

    assert_eq!(request.email.as_deref(), Some("ana@portmaster.local"));
    assert_eq!(request.password.as_deref(), Some("Portmaster1"));
}

#[test]
fn corpo_ilegivel_vira_400_e_nao_panico() {
    let error = FlatBuffersDecodeStrategy
        .decode::<LoginXRequest>(b"nao e um flatbuffer")
        .expect_err("lixo não é buffer");

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn corpo_vazio_tambem_e_ilegivel() {
    let error = FlatBuffersDecodeStrategy
        .decode::<LoginXRequest>(b"")
        .expect_err("corpo vazio não é buffer");

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
}
