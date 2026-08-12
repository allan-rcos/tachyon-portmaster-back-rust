//! Os testes de `api_response`.

use super::*;
use axum::http::header;
use pretty_assertions::assert_eq;

fn body() -> ProblemX {
    ProblemX {
        kind: "about:blank",
        title: "OK".to_owned(),
        status: 200,
        detail: "tudo certo".to_owned(),
    }
}

#[tokio::test]
async fn o_corpo_de_acerto_sai_codificado() {
    let response = ApiResponse::ok(Ok(body())).into_response();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(header::CONTENT_TYPE));
}

/// O erro sai pelo mesmo caminho do acerto, e com o status dele.
#[tokio::test]
async fn o_erro_carrega_o_proprio_status() {
    let response = ApiResponse::<ProblemX>::ok(Err(ApiError::unauthenticated())).into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
