//! O corpo que não passa pela negociação.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;

use crate::error::api_error::ApiError;

/// Teto do corpo de uma requisição.
///
/// O mesmo que o [`Body`](crate::wire::body::Body) negociado aplica. Está
/// repetido em vez de compartilhado porque os dois extractors são caminhos
/// independentes, e um `const` público só para uni-los criaria um export a mais
/// num módulo que já exporta o seu tipo.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Um corpo lido **sempre** como JSON, qualquer que seja o `Content-Type`.
///
/// Existe para um endpoint só: `PUT /users/{id}/roles`. O payload dele nunca
/// entrou nos `.fbs` — o PHP o lia com `json_decode` direto do corpo, sem tabela
/// e sem negociação — e os `.fbs` são contrato publicado com os clientes, não
/// algo a estender por conveniência nossa. Enquanto a lista de papéis não tiver
/// tabela própria, ler JSON aqui é o que mantém o endpoint respondendo ao mesmo
/// corpo de antes.
///
/// O `Content-Type` é ignorado de propósito: o cliente de referência anuncia
/// `FlatBuffers` em toda requisição e mesmo assim manda JSON **neste** corpo, que
/// é exatamente a situação que o PHP tolerava.
#[derive(Debug)]
pub struct JsonBody<T>(pub(crate) T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::unreadable_body(format!("não foi possível ler o corpo: {e}")))?;

        if bytes.len() > MAX_BODY_BYTES {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "corpo grande demais",
            ));
        }

        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|e| ApiError::unreadable_body(format!("corpo JSON inválido: {e}")))
    }
}
