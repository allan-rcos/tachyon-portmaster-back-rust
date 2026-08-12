//! Os testes de `encode_context`.

use super::*;
use crate::wire::vo::common::problem_x::ProblemX;
use pretty_assertions::assert_eq;

fn problem() -> ProblemX {
    ProblemX {
        kind: "about:blank",
        title: "Not Acceptable".to_owned(),
        status: 406,
        detail: "nada a dizer".to_owned(),
    }
}

#[tokio::test]
async fn a_resposta_sai_no_formato_do_escopo() {
    let response = EncodeContext::scope(MediaType::FlatBuffers, async {
        EncodeContext.respond(StatusCode::OK, &problem())
    })
    .await;

    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or_default()),
        Some(MediaType::FlatBuffers.header_value())
    );
}

/// Um erro que nasce antes do middleware ainda precisa virar resposta.
#[tokio::test]
async fn fora_do_escopo_a_resposta_sai_em_json() {
    let response = EncodeContext.respond(StatusCode::OK, &problem());

    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or_default()),
        Some(MediaType::Json.header_value())
    );
}

#[tokio::test]
async fn escopos_de_requisicoes_diferentes_nao_se_misturam() {
    let json = tokio::spawn(EncodeContext::scope(MediaType::Json, async {
        EncodeContext.respond(StatusCode::OK, &problem())
    }));
    let binary = tokio::spawn(EncodeContext::scope(MediaType::FlatBuffers, async {
        EncodeContext.respond(StatusCode::OK, &problem())
    }));

    let (json, binary) = tokio::join!(json, binary);

    assert_eq!(
        json.expect("a tarefa não entra em pânico")
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or_default()),
        Some(MediaType::Json.header_value())
    );
    assert_eq!(
        binary
            .expect("a tarefa não entra em pânico")
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or_default()),
        Some(MediaType::FlatBuffers.header_value())
    );
}
