//! Os testes de `api_error`.

use super::*;
use crate::wire::media_type::MediaType;
use axum::http::header;
use pretty_assertions::assert_eq;

/// O slug vai para o log.
///
/// No corpo, ele descreveria ao cliente o mapa de autorização do sistema.
#[test]
fn a_permissao_negada_nao_vaza_o_slug_no_corpo() {
    let error = ApiError::of_app(AppError::PermissionDenied {
        permission: "container:seal",
    });

    assert_eq!(error.status(), StatusCode::FORBIDDEN);
    assert!(
        !error.detail.contains("container:seal"),
        "o corpo não deveria nomear a permissão: {}",
        error.detail
    );
}

#[test]
fn a_falha_de_infra_nao_vaza_o_motivo() {
    let error = ApiError::of_app(AppError::Infra(anyhow::anyhow!(
        "Connection refused (os error 111) para db:3306"
    )));

    assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        !error.detail.contains("db:3306"),
        "topologia vazou: {}",
        error.detail
    );
}

/// O domínio acumula; perder campos aqui obrigaria o cliente a descobrir
/// um problema por requisição.
#[test]
fn a_validacao_lista_todos_os_campos() {
    let error = ApiError::of_app(AppError::Validation(vec![
        FieldError::new("email", "malformado"),
        FieldError::new("password", "curta demais"),
    ]));

    assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(error.detail.contains("email"));
    assert!(error.detail.contains("password"));
}

/// O slug negado não aparece no corpo, mas o 403 aparece.
#[test]
fn a_permissao_que_falta_e_proibido() {
    let error = ApiError::of_app(AppError::permission_denied("container:dispatch"));

    assert_eq!(error.status(), StatusCode::FORBIDDEN);
    assert!(
        !error.detail.contains("container:dispatch"),
        "o slug descreve o mapa de autorização para quem acabou de ser recusado"
    );
}

/// Sem requisição de onde negociar, vale o padrão — e o padrão é JSON.
#[test]
fn o_erro_sem_negociacao_sai_em_json() {
    let response = ApiError::unauthenticated().into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
}

/// É o item que motivou o desenho: um cliente que fala `FlatBuffers` recebe
/// o erro em `FlatBuffers`, e não num JSON que ele não sabe ler.
///
/// O erro não carrega mais o formato — ele o encontra no escopo, de onde
/// quer que tenha nascido.
#[tokio::test]
async fn o_erro_sai_no_formato_que_o_cliente_pediu() {
    let response = EncodeContext::scope_for_test(MediaType::FlatBuffers, async {
        ApiError::unauthenticated().into_response()
    })
    .await;

    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/x-flatbuffers")
    );
}
