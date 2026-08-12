//! Os testes de `media_type`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn sem_content_type_o_corpo_e_binario() {
    assert_eq!(
        MediaType::of_request(None).expect("ausente é o formato nativo"),
        MediaType::FlatBuffers
    );
    assert_eq!(
        MediaType::of_request(Some(FLATBUFFERS)).expect("é o formato nativo"),
        MediaType::FlatBuffers
    );
    assert_eq!(
        MediaType::of_request(Some("application/json; charset=utf-8")).expect("é json"),
        MediaType::Json
    );
}

/// Ele anunciou alguma coisa, e não foi nenhuma das duas que lemos.
#[test]
fn um_content_type_desconhecido_e_recusado() {
    assert!(MediaType::of_request(Some("application/xml")).is_err());
}

#[test]
fn sem_accept_a_resposta_e_legivel() {
    assert_eq!(
        MediaType::of_response(None).expect("ausente é json"),
        MediaType::Json
    );
    assert_eq!(
        MediaType::of_response(Some("*/*")).expect("curinga é json"),
        MediaType::Json
    );
    assert_eq!(
        MediaType::of_response(Some(FLATBUFFERS)).expect("é o formato nativo"),
        MediaType::FlatBuffers
    );
    assert_eq!(
        MediaType::of_response(Some(OCTET_STREAM)).expect("binário genérico"),
        MediaType::FlatBuffers
    );
}

#[test]
fn o_accept_com_qualidade_e_lista_ainda_e_entendido() {
    // Um navegador manda algo como `text/html,application/json;q=0.9,*/*`.
    assert_eq!(
        MediaType::of_response(Some("text/html,application/json;q=0.9,*/*;q=0.8"))
            .expect("a lista nomeia json"),
        MediaType::Json
    );
}

/// O curinga tem que ganhar do tipo que não sabemos escrever, senão todo
/// navegador que abrisse a API levaria 406.
#[test]
fn o_curinga_salva_o_accept_que_so_nomeia_o_que_nao_escrevemos() {
    assert_eq!(
        MediaType::of_response(Some("text/html,application/xhtml+xml,*/*;q=0.8"))
            .expect("o curinga está lá"),
        MediaType::Json
    );
}

#[test]
fn um_accept_concreto_e_desconhecido_e_recusado() {
    assert!(MediaType::of_response(Some("application/xml")).is_err());
    assert!(MediaType::of_response(Some("text/html")).is_err());
}
