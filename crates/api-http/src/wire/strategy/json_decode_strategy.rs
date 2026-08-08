//! A leitura de um corpo JSON.

use serde_json::Value;

use crate::error::api_error::ApiError;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::strategy::decode_strategy::DecodeStrategy;

/// Lê o corpo como JSON e entrega o objeto à factory.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JsonDecodeStrategy;

impl DecodeStrategy for JsonDecodeStrategy {
    fn decode<F: RequestFactory>(&self, bytes: &[u8]) -> Result<F::Message, ApiError> {
        // A mensagem do serde nomeia a linha e a coluna, e devolvê-la ajuda quem
        // está integrando. Não há segredo nela: o corpo é o que o próprio
        // cliente mandou.
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo JSON inválido: {e}")))?;

        // Um corpo que não é objeto não tem campo nenhum para a factory ler. É
        // erro de forma, não de conteúdo — e por isso não vira 422.
        let Value::Object(source) = value else {
            return Err(ApiError::unreadable_body(
                "o corpo JSON precisa ser um objeto",
            ));
        };

        F::from_json(&source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::dto::auth::login_request_factory::LoginRequestFactory;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_corpo_json_volta_a_ser_a_mesma_mensagem() {
        let request = JsonDecodeStrategy
            .decode::<LoginRequestFactory>(
                br#"{"email":"ana@portmaster.local","password":"Portmaster1"}"#,
            )
            .expect("o corpo é JSON válido");

        assert_eq!(request.email.as_deref(), Some("ana@portmaster.local"));
        assert_eq!(request.password.as_deref(), Some("Portmaster1"));
    }

    #[test]
    fn corpo_ilegivel_vira_400_e_nao_panico() {
        // Um cliente que manda lixo não deve derrubar o handler. E é 400, não o
        // 404 do PHP: 404 é ausência de recurso, afirmada pela suíte Go em cinco
        // pontos, e colidir os dois apaga a diferença.
        let error = JsonDecodeStrategy
            .decode::<LoginRequestFactory>(b"{{{")
            .expect_err("lixo não é JSON");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn corpo_vazio_tambem_e_ilegivel() {
        // Ausência é um caso de ilegibilidade, e não um caso à parte.
        let error = JsonDecodeStrategy
            .decode::<LoginRequestFactory>(b"")
            .expect_err("corpo vazio não é objeto");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn um_json_que_nao_e_objeto_e_recusado() {
        // Um array não tem campo nenhum para a factory ler: é erro de forma, e
        // por isso 400 e não o 422 de conteúdo.
        let error = JsonDecodeStrategy
            .decode::<LoginRequestFactory>(b"[1,2,3]")
            .expect_err("um array não tem campos");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn um_campo_ausente_chega_como_none_e_nao_como_erro() {
        // É a mudança que o desenho traz: o DTO recebe `None`, o `TableModule`
        // recusa nomeando **todos** os campos que faltaram, e o cliente ganha um
        // 422 útil em vez de um 400 genérico do serde.
        let request = JsonDecodeStrategy
            .decode::<LoginRequestFactory>(br#"{"email":"ana@portmaster.local"}"#)
            .expect("faltar campo não é erro de formato");

        assert_eq!(request.password, None);
    }
}
