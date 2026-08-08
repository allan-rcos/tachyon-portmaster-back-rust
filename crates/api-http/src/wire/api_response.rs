//! A resposta de um handler, antes de virar bytes.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::wire::factory::renderable::Renderable;
use crate::wire::wire::Wire;

/// Uma resposta pronta para ser escrita no formato negociado.
///
/// Carrega status, corpo e cookies juntos porque as três coisas saem juntas —
/// separá-las obrigaria cada handler a montar uma `Response` à mão.
///
/// O corpo é `Box<dyn Renderable>` e não um genérico: um handler com dois
/// caminhos de retorno devolve tabelas diferentes, e generificar `ApiResponse`
/// obrigaria cada um a virar um `enum`. É o segundo e último ponto dinâmico do
/// wire — o primeiro é a strategy dentro do [`Wire`].
pub(crate) struct ApiResponse {
    /// O formato negociado desta requisição.
    wire: Wire,
    /// O status HTTP da resposta.
    status: StatusCode,
    /// O corpo já pronto para ser escrito, com o tipo apagado.
    body: Box<dyn Renderable>,
    /// Os `Set-Cookie` a acrescentar na resposta, um cabeçalho por entrada.
    cookies: Vec<String>,
}

impl ApiResponse {
    /// Uma resposta `200` com corpo.
    pub(crate) fn ok(wire: Wire, body: impl Renderable + 'static) -> Self {
        Self::with_status(wire, StatusCode::OK, body)
    }

    /// Uma resposta `201` com corpo.
    pub(crate) fn created(wire: Wire, body: impl Renderable + 'static) -> Self {
        Self::with_status(wire, StatusCode::CREATED, body)
    }

    /// Uma resposta com corpo e status escolhido.
    pub(crate) fn with_status(
        wire: Wire,
        status: StatusCode,
        body: impl Renderable + 'static,
    ) -> Self {
        Self {
            wire,
            status,
            body: Box::new(body),
            cookies: Vec::new(),
        }
    }

    /// Acrescenta um `Set-Cookie`.
    #[must_use]
    pub(crate) fn with_cookie(mut self, cookie: String) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl IntoResponse for ApiResponse {
    /// Escreve o corpo no formato negociado, e os cookies.
    ///
    /// Falhar ao escrever a **própria** resposta é defeito nosso, não do
    /// cliente. O 502 que o `ApiError::unrenderable` produz distingue isso do
    /// 500 que o middleware `Recover` devolve quando algo entrou em pânico — e a
    /// diferença importa para quem lê o painel.
    fn into_response(self) -> Response {
        let encode = self.wire.encode();

        let mut response = match encode.encode(self.body.as_ref()) {
            Ok(bytes) => (
                self.status,
                [(header::CONTENT_TYPE, encode.content_type())],
                bytes,
            )
                .into_response(),
            Err(error) => error.into_response(),
        };

        for cookie in self.cookies {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::api_error::ApiError;
    use crate::wire::factory::response_factory::ResponseFactory;
    use crate::wire::media_type::MediaType;
    use crate::wire::strategy::encode_strategy::EncodeStrategy as _;
    use crate::wire::strategy::flatbuffers_encode_strategy::FlatBuffersEncodeStrategy;
    use crate::wire::strategy::json_encode_strategy::JsonEncodeStrategy;
    use crate::wire::tables as fbs;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    /// Uma factory qualquer: o que se testa aqui é o envelope, não a mensagem.
    struct ProductFactory;

    impl ResponseFactory for ProductFactory {
        type Table = fbs::product::ProductResponse;

        fn table(&self) -> Result<Self::Table, ApiError> {
            Ok(fbs::product::ProductResponse {
                id: Some("aZ3".into()),
                name: Some("Cimento".into()),
                density: 1.44,
                risk_class: fbs::common::RiskClass::None,
            })
        }
    }

    fn json() -> Wire {
        Wire::new(MediaType::Json, Arc::new(JsonEncodeStrategy))
    }

    fn flatbuffers() -> Wire {
        Wire::new(MediaType::FlatBuffers, Arc::new(FlatBuffersEncodeStrategy))
    }

    /// O `Content-Type` que sai é o da strategy que escreveu os bytes, e não
    /// uma segunda decisão tomada aqui — é o que garante que os dois nunca
    /// divirjam.
    #[test]
    fn a_resposta_anuncia_o_formato_em_que_saiu() {
        let text = ApiResponse::ok(json(), ProductFactory).into_response();
        let binary = ApiResponse::ok(flatbuffers(), ProductFactory).into_response();

        assert_eq!(
            text.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(JsonEncodeStrategy.content_type())
        );
        assert_eq!(
            binary
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(FlatBuffersEncodeStrategy.content_type())
        );
    }

    #[test]
    fn a_criacao_responde_201() {
        let response = ApiResponse::created(json(), ProductFactory).into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[test]
    fn os_cookies_saem_em_cabecalhos_separados() {
        // Dois `Set-Cookie` num cabeçalho só não são lidos por navegador nenhum.
        let response = ApiResponse::ok(json(), ProductFactory)
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
}
