//! O corpo da requisição, já na mensagem que a factory descreve.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;

use crate::error::api_error::ApiError;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::decode_strategy::DecodeStrategy;
use crate::wire::strategy::flatbuffers_decode_strategy::FlatBuffersDecodeStrategy;
use crate::wire::strategy::json_decode_strategy::JsonDecodeStrategy;
use crate::wire::wire::Wire;

/// Teto do corpo de uma requisição.
///
/// Nenhum endpoint desta API recebe payload grande — o maior é uma lista de ids
/// de papel. O limite existe para que um corpo enorme seja recusado antes de ser
/// lido inteiro na memória, e não depois.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// O corpo da requisição, lido pela factory `F`.
///
/// O parâmetro é a **factory**, não a mensagem: é ela que sabe ler os dois
/// formatos, e pedi-la por tipo é o que mantém a rota dizendo qual mensagem
/// espera — `Body<LoginRequestFactory>` — sem instanciar nada.
pub(crate) struct Body<F: RequestFactory>(pub(crate) F::Message);

impl<S, F> FromRequest<S> for Body<F>
where
    F: RequestFactory,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();
        let wire = Wire::from_request_parts(&mut parts, state).await?;
        let request = Request::from_parts(parts, body);

        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::unreadable_body(format!("não foi possível ler o corpo: {e}")))?;

        if bytes.len() > MAX_BODY_BYTES {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "corpo grande demais",
            ));
        }

        // O `match` fica aqui, e não num `dyn`: `DecodeStrategy::decode` é
        // genérico sobre a factory e por isso a trait não é object-safe — o que
        // é conveniente, porque mantém o caminho de requisição estático.
        let message = match wire.request() {
            MediaType::Json => JsonDecodeStrategy.decode::<F>(&bytes),
            MediaType::FlatBuffers => FlatBuffersDecodeStrategy.decode::<F>(&bytes),
        }?;

        Ok(Self(message))
    }
}

use axum::extract::FromRequestParts as _;
