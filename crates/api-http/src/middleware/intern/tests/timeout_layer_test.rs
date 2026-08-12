//! Os testes de `timeout_layer`.

use super::*;
use axum::response::IntoResponse as _;
use tower::{ServiceBuilder, ServiceExt};

#[tokio::test]
async fn o_que_demora_demais_vira_504() {
    let service = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_millis(10)))
        .service_fn(|_: Request| async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok::<_, std::convert::Infallible>(Response::new(axum::body::Body::empty()))
        });

    let response = service
        .oneshot(Request::new(axum::body::Body::empty()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
}

#[tokio::test]
async fn o_que_responde_a_tempo_atravessa() {
    let service = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .service_fn(|_: Request| async move {
            Ok::<_, std::convert::Infallible>((StatusCode::CREATED, "rápido").into_response())
        });

    let response = service
        .oneshot(Request::new(axum::body::Body::empty()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
