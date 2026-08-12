//! Os testes de `recover_layer`.

use super::*;
use axum::response::IntoResponse as _;
use tower::{ServiceBuilder, ServiceExt};

#[tokio::test]
async fn um_panico_vira_500_e_nao_derruba_o_servidor() {
    let service =
        ServiceBuilder::new()
            .layer(RecoverLayer::new())
            .service_fn(|_: Request| async move {
                panic!("algo explodiu no handler");
                #[allow(unreachable_code, reason = "o panic! acima é o assunto do teste")]
                Ok::<_, std::convert::Infallible>(Response::new(axum::body::Body::empty()))
            });

    let response = service
        .oneshot(Request::new(axum::body::Body::empty()))
        .await
        .expect("o middleware não deveria propagar o pânico");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn a_resposta_normal_atravessa_intacta() {
    let service =
        ServiceBuilder::new()
            .layer(RecoverLayer::new())
            .service_fn(|_: Request| async move {
                Ok::<_, std::convert::Infallible>(
                    (StatusCode::CREATED, "tudo certo").into_response(),
                )
            });

    let response = service
        .oneshot(Request::new(axum::body::Body::empty()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
