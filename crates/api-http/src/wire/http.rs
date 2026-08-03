//! A ponte entre o axum e o wire negociado.
//!
//! Do lado da entrada, [`Body`] é um extractor que lê o corpo no formato que o
//! `Content-Type` anunciou. Do lado da saída, [`Negotiated`] é uma resposta que
//! se serializa no formato que o `Accept` pediu.
//!
//! Nenhum handler toca `serde_json` ou `planus::Builder`: ele recebe uma tabela
//! e devolve uma tabela. Qual formato atravessou o fio é uma pergunta que só
//! estes dois tipos fazem.

use axum::body::Bytes;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use super::negotiate::{decode, encode, MediaType, WireRequest, WireResponse};
use crate::error::ApiError;

/// Teto do corpo de uma requisição.
///
/// Nenhum endpoint desta API recebe payload grande — o maior é uma lista de ids
/// de papel. O limite existe para que um corpo enorme seja recusado antes de ser
/// lido inteiro na memória, e não depois.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// O corpo da requisição, já na tabela pedida.
#[derive(Debug)]
pub(crate) struct Body<R>(pub(crate) R);

impl<S, R> FromRequest<S> for Body<R>
where
    R: WireRequest,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let media = MediaType::of_request(header_str(request.headers(), &header::CONTENT_TYPE));

        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::malformed_body(format!("não foi possível ler o corpo: {e}")))?;

        if bytes.len() > MAX_BODY_BYTES {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "corpo grande demais",
            ));
        }

        Ok(Self(decode(media, &bytes)?))
    }
}

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
/// FlatBuffers em toda requisição e mesmo assim manda JSON **neste** corpo, que
/// é exatamente a situação que o PHP tolerava.
#[derive(Debug)]
pub(crate) struct JsonBody<T>(pub(crate) T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::malformed_body(format!("não foi possível ler o corpo: {e}")))?;

        if bytes.len() > MAX_BODY_BYTES {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "corpo grande demais",
            ));
        }

        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|e| ApiError::malformed_body(format!("corpo JSON inválido: {e}")))
    }
}

/// O formato que a resposta deve ter, lido do `Accept`.
///
/// Extractor separado porque um handler pode precisar dele **sem** ter corpo de
/// entrada — todo `GET` está nesse caso.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Accept(pub(crate) MediaType);

impl<S: Send + Sync> FromRequestParts<S> for Accept {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(MediaType::of_response(header_str(
            &parts.headers,
            &header::ACCEPT,
        ))))
    }
}

/// Uma resposta que se codifica no formato negociado.
///
/// Carrega também o status e os cookies, porque as três coisas saem juntas e
/// separá-las obrigaria cada handler a montar uma `Response` à mão.
#[derive(Debug)]
pub(crate) struct Negotiated<T> {
    media: MediaType,
    status: StatusCode,
    table: Option<T>,
    cookies: Vec<String>,
}

impl<T: WireResponse> Negotiated<T> {
    /// Uma resposta `200` com corpo.
    pub(crate) fn ok(accept: Accept, table: T) -> Self {
        Self {
            media: accept.0,
            status: StatusCode::OK,
            table: Some(table),
            cookies: Vec::new(),
        }
    }

    /// Uma resposta `201` com corpo.
    pub(crate) fn created(accept: Accept, table: T) -> Self {
        Self {
            media: accept.0,
            status: StatusCode::CREATED,
            table: Some(table),
            cookies: Vec::new(),
        }
    }

    /// Acrescenta um `Set-Cookie`.
    pub(crate) fn with_cookie(mut self, cookie: String) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl<T: WireResponse> IntoResponse for Negotiated<T> {
    fn into_response(self) -> Response {
        let mut response = match self.table {
            None => (self.status, Vec::new()).into_response(),
            Some(table) => match encode(self.media, &table) {
                Ok(body) => (
                    self.status,
                    [(header::CONTENT_TYPE, self.media.header_value())],
                    body,
                )
                    .into_response(),
                // Falhar ao serializar a **própria** resposta é defeito nosso,
                // não do cliente: o 500 é honesto e o motivo vai para o log.
                Err(error) => error.into_response(),
            },
        };

        for cookie in self.cookies {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }

        response
    }
}

/// Uma resposta sem corpo, que ainda pode carregar cookies.
///
/// `204` é o que o PHP devolvia em refresh, logout e nas operações cujo
/// resultado é o próprio estado ter mudado.
#[derive(Debug, Default)]
pub(crate) struct NoContent {
    cookies: Vec<String>,
}

impl NoContent {
    /// Uma resposta `204` vazia.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Acrescenta um `Set-Cookie`.
    pub(crate) fn with_cookie(mut self, cookie: String) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        let mut response = StatusCode::NO_CONTENT.into_response();

        for cookie in self.cookies {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }

        response
    }
}

/// O valor de um cabeçalho como texto, se legível.
fn header_str<'a>(headers: &'a HeaderMap, name: &header::HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::tables as fbs;
    use pretty_assertions::assert_eq;

    fn product() -> fbs::product::ProductResponse {
        fbs::product::ProductResponse {
            id: Some("aZ3".into()),
            name: Some("Cimento".into()),
            density: 1.44,
            risk_class: fbs::common::RiskClass::None,
        }
    }

    #[test]
    fn a_resposta_anuncia_o_formato_em_que_saiu() {
        let json = Negotiated::ok(Accept(MediaType::Json), product()).into_response();
        let binary = Negotiated::ok(Accept(MediaType::FlatBuffers), product()).into_response();

        assert_eq!(
            json.headers().get(header::CONTENT_TYPE).unwrap(),
            crate::wire::negotiate::JSON
        );
        assert_eq!(
            binary.headers().get(header::CONTENT_TYPE).unwrap(),
            crate::wire::negotiate::FLATBUFFERS
        );
    }

    #[test]
    fn os_cookies_saem_em_cabecalhos_separados() {
        // Dois `Set-Cookie` num cabeçalho só não são lidos por navegador nenhum.
        let response = Negotiated::ok(Accept(MediaType::Json), product())
            .with_cookie("auth_token=a; Path=/".into())
            .with_cookie("refresh_token=b; Path=/".into())
            .into_response();

        assert_eq!(
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .count(),
            2
        );
    }

    #[test]
    fn a_criacao_responde_201() {
        let response = Negotiated::created(Accept(MediaType::Json), product()).into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[test]
    fn a_resposta_vazia_ainda_carrega_cookies() {
        // É o caso do logout: nada a dizer, mas há cookies a limpar.
        let response = NoContent::new()
            .with_cookie("auth_token=; Max-Age=0".into())
            .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .count(),
            1
        );
    }
}
