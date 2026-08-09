//! O corpo da requisição, já no VO da mensagem.

use axum::body::Bytes;
use axum::extract::{FromRequest, FromRequestParts as _, Request};
use axum::http::{header, StatusCode};

use crate::error::api_error::ApiError;
use crate::wire::decoder::Decoder;
use crate::wire::encoder::Encoder;
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
/// um DTO, não uma factory. Qual dos dois formatos chegou é assunto do
/// [`Decoder`], que resolve isso e some.
pub(crate) struct Body<X: RequestX>(pub(crate) X);

impl<S, X> FromRequest<S> for Body<X>
where
    X: RequestX,
    S: Send + Sync,
{
    type Rejection = ApiError;

    /// Lê o corpo e o entrega ao decoder do formato anunciado.
    ///
    /// O encoder é extraído antes de qualquer coisa poder falhar, e anexado à
    /// recusa: é o que faz um corpo ilegível sair no formato que o cliente
    /// pediu, em vez de num JSON que ele talvez não leia.
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();

        let encoder = Encoder::from_request_parts(&mut parts, state)
            .await
            .unwrap_or_default();
        let decoder = Decoder::of_request(
            parts
                .headers
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
        );

        let bytes = Bytes::from_request(Request::from_parts(parts, body), state)
            .await
            .map_err(|e| {
                ApiError::unreadable_body(format!("não foi possível ler o corpo: {e}"))
                    .with_encoder(encoder)
            })?;

        if bytes.len() > MAX_BODY_BYTES {
            return Err(
                ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "corpo grande demais")
                    .with_encoder(encoder),
            );
        }

        decoder
            .decode(&bytes)
            .map(Self)
            .map_err(|error| error.with_encoder(encoder))
    }
}
