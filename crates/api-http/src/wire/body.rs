//! O corpo da requisição, já no VO da mensagem.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;

use crate::middleware::decode_port::DecodePort as _;
use crate::middleware::intern::decode_context::DecodeContext;
use crate::ports::error::api_error::ApiError;
use crate::wire::x::request_x::RequestX;

/// O teto de um corpo de requisição.
///
/// Um megabyte cobre com folga a maior mensagem que o `.fbs` descreve. O limite
/// existe para que um cliente não consiga fazer o processo alocar sem limite
/// mandando um `Content-Length` grande.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Extrai o corpo no VO da mensagem, seja qual for o formato que chegou.
///
/// A rota escreve `Body<LoginXRequest>` e recebe um `LoginXRequest`: o VO, não
/// um DTO, não uma factory.
///
/// Ele **não decide** o formato. Quem decidiu foi o middleware, uma vez, no
/// começo da requisição; aqui só se aplica o que ele escolheu. Reler o
/// `Content-Type` aqui, montar um decoder e extrair um encoder de reserva para
/// a recusa seriam três decisões de negociação dentro de um extractor de corpo.
pub(crate) struct Body<X: RequestX>(pub(crate) X);

impl<S, X> FromRequest<S> for Body<X>
where
    X: RequestX,
    S: Send + Sync,
{
    type Rejection = ApiError;

    /// Lê o corpo e o entrega à strategy que o escopo escolheu.
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

        DecodeContext.decode(&bytes).map(Self)
    }
}
