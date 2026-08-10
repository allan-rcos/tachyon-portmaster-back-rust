//! O contexto do Strategy pattern na saída.

use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse as _, Response};
use cookie::Cookie;
use std::convert::Infallible;

use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy as _;
use crate::wire::strategy::flatbuffers_encode_strategy::FlatBuffersEncodeStrategy;
use crate::wire::strategy::json_encode_strategy::JsonEncodeStrategy;
use crate::wire::x::response_x::ResponseX;

/// A strategy da vez, e nada além dela.
///
/// É o campo mutável do contexto — o que o Strategy pattern manda ser mutável.
/// As strategies em si são ZSTs imóveis: trocar de formato é trocar a variante,
/// não realocar nada.
///
/// O enum é **privado ao módulo** de propósito. É o que garante que ninguém de
/// fora consiga perguntar qual formato está em uso: só o contexto e o setter
/// dele sabem, e o [`Encoder::content_type`] logo abaixo é o único ponto do
/// sistema em que essa informação vira um cabeçalho.
#[derive(Debug, Clone, Copy)]
enum Strategy {
    /// Escreve JSON.
    Json(JsonEncodeStrategy),
    /// Escreve `FlatBuffers`.
    FlatBuffers(FlatBuffersEncodeStrategy),
}

/// Quem escreve a resposta no formato que a requisição pediu.
///
/// O contexto do Strategy pattern: guarda a strategy corrente e delega. Quem
/// tem um `Encoder` na mão consegue **responder**, e é só isso — não consegue
/// descobrir em que formato, nem escolher outro sem passar pelo
/// [`Encoder::set`].
///
/// ## Por que ele não atravessa a aplicação
///
/// Ele nasce como extractor, no adaptador de rota, e morre ao virar
/// [`Response`]. O controller não o recebe, não o guarda e não o repassa: o
/// controller devolve um VO e não sabe que existe negociação de conteúdo.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Encoder {
    /// A strategy em uso.
    current: Strategy,
}

impl Encoder {
    /// O encoder para o que o `Accept` pediu.
    pub(crate) fn of_response(accept: Option<&str>) -> Self {
        let mut encoder = Self {
            current: Strategy::Json(JsonEncodeStrategy),
        };
        encoder.set(MediaType::of_response(accept));

        encoder
    }

    /// O encoder para os cabeçalhos de uma requisição.
    ///
    /// Existe para os middlewares, que têm a requisição inteira em mãos mas não
    /// passam por extractor nenhum — é como o `Recover` e o `Timeout` respondem
    /// no formato certo em vez de despejar um corpo fixo.
    pub(crate) fn of_headers(headers: &HeaderMap) -> Self {
        Self::of_response(
            headers
                .get(header::ACCEPT)
                .and_then(|value| value.to_str().ok()),
        )
    }

    /// Troca a strategy corrente.
    pub(crate) const fn set(&mut self, media: MediaType) {
        self.current = match media {
            MediaType::Json => Strategy::Json(JsonEncodeStrategy),
            MediaType::FlatBuffers => Strategy::FlatBuffers(FlatBuffersEncodeStrategy),
        };
    }

    /// A resposta completa: corpo codificado, tipo de mídia e cookies.
    ///
    /// É o único jeito de tirar bytes de um `Encoder`, e é de propósito: se
    /// codificar e carimbar o cabeçalho fossem duas chamadas, existiria um
    /// caminho em que a primeira acontece sem a segunda — um corpo saindo com o
    /// tipo de mídia errado, que é exatamente o defeito que este desenho fecha.
    ///
    /// Falha na serialização vira 502 e sai como corpo de problema — pelo mesmo
    /// encoder, para que nem o erro do erro escape da negociação.
    pub(crate) fn respond<X: ResponseX>(
        self,
        status: StatusCode,
        body: &X,
        cookies: Vec<Cookie<'static>>,
    ) -> Response {
        let encoded = match self.current {
            Strategy::Json(strategy) => strategy.encode(body),
            Strategy::FlatBuffers(strategy) => strategy.encode(body),
        };

        let mut response = match encoded {
            Ok(bytes) => {
                (status, [(header::CONTENT_TYPE, self.content_type())], bytes).into_response()
            }
            Err(error) => error.into_response(),
        };

        let headers = response.headers_mut();
        for cookie in cookies {
            if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
                headers.append(header::SET_COOKIE, value);
            }
        }

        response
    }

    /// O tipo de mídia que a strategy corrente escreve.
    ///
    /// Privado, e o único consumidor é o [`Self::respond`] logo acima: o
    /// contexto sabe o que escolheu, ninguém mais precisa saber.
    const fn content_type(self) -> &'static str {
        match self.current {
            Strategy::Json(_) => MediaType::Json.header_value(),
            Strategy::FlatBuffers(_) => MediaType::FlatBuffers.header_value(),
        }
    }
}

impl Default for Encoder {
    /// JSON.
    ///
    /// É o mesmo destino de um `Accept` que não reconhecemos, e vale para os
    /// erros que nascem antes de haver requisição de onde negociar.
    fn default() -> Self {
        Self {
            current: Strategy::Json(JsonEncodeStrategy),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Encoder {
    /// Nunca recusa: um `Accept` que não reconhecemos cai no formato padrão.
    ///
    /// Recusar seria responder que não sabemos responder — e teríamos que
    /// escolher um formato para dizer isso, que é a escolha que acabamos de
    /// declarar impossível.
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::of_headers(&parts.headers))
    }
}
