//! A leitura de um corpo JSON.

use crate::ports::error::api_error::ApiError;
use crate::wire::strategy::decode_strategy::DecodeStrategy;
use crate::wire::x::request_x::RequestX;

/// Lê o corpo como o DTO de JSON da mensagem.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JsonDecodeStrategy;

impl DecodeStrategy for JsonDecodeStrategy {
    /// Desserializa direto no DTO da mensagem.
    ///
    /// O `derive` do serde gera este código no build; antes o corpo virava um
    /// `Map<String, Value>` e cada factory pescava campo a campo em tempo de
    /// execução, sem que o compilador soubesse de nada. O tipo agora é o
    /// contrato, e um campo renomeado no DTO quebra a compilação em vez de
    /// virar `None` silencioso.
    ///
    /// A mensagem do serde nomeia linha e coluna, e devolvê-la ajuda quem está
    /// integrando — não há segredo nela, o corpo é o que o cliente mandou.
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError> {
        let dto: X::Json = serde_json::from_slice(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo JSON inválido: {e}")))?;

        Ok(X::of_json(dto))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::vo::auth::login_x_request::LoginXRequest;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_corpo_json_volta_a_ser_a_mesma_mensagem() {
        let request: LoginXRequest = JsonDecodeStrategy
            .decode(br#"{"email":"ana@portmaster.local","password":"Portmaster1"}"#)
            .expect("o corpo é JSON válido");

        assert_eq!(request.email.as_deref(), Some("ana@portmaster.local"));
        assert_eq!(request.password.as_deref(), Some("Portmaster1"));
    }

    /// Um cliente que manda lixo não deve derrubar o controller.
    ///
    /// E é 400, não o 404 do PHP: 404 é ausência de recurso, afirmada pela
    /// suíte Go em cinco pontos, e colidir os dois apaga a diferença.
    #[test]
    fn corpo_ilegivel_vira_400_e_nao_panico() {
        let error = JsonDecodeStrategy
            .decode::<LoginXRequest>(b"{{{")
            .expect_err("lixo não é JSON");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn corpo_vazio_tambem_e_ilegivel() {
        // Ausência é um caso de ilegibilidade, e não um caso à parte.
        let error = JsonDecodeStrategy
            .decode::<LoginXRequest>(b"")
            .expect_err("corpo vazio não é objeto");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    /// Um array não tem campo nenhum para o DTO ler: é erro de forma, e por
    /// isso 400 e não o 422 de conteúdo.
    #[test]
    fn um_json_que_nao_e_objeto_e_recusado() {
        let error = JsonDecodeStrategy
            .decode::<LoginXRequest>(b"[1,2,3]")
            .expect_err("um array não tem campos");

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    /// É o ponto do desenho: o VO recebe `None`, o `TableModule` recusa
    /// nomeando **todos** os campos que faltaram, e o cliente ganha um 422 útil
    /// em vez de um 400 genérico do serde.
    #[test]
    fn um_campo_ausente_chega_como_none_e_nao_como_erro() {
        let request: LoginXRequest = JsonDecodeStrategy
            .decode(br#"{"email":"ana@portmaster.local"}"#)
            .expect("faltar campo não é erro de formato");

        assert_eq!(request.password, None);
    }
}
