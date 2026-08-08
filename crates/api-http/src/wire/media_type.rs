//! Qual dos dois formatos está em jogo.

/// `application/json`.
const JSON: &str = "application/json";

/// O tipo nativo desta API.
const FLATBUFFERS: &str = "application/x-flatbuffers";

/// Qual dos dois formatos está em jogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MediaType {
    /// Texto, para depuração e clientes genéricos.
    Json,
    /// Binário, o formato nativo.
    FlatBuffers,
}

impl MediaType {
    /// O formato do **corpo que chegou**, pelo `Content-Type`.
    ///
    /// Cabeçalho ausente ou irreconhecível cai em `FlatBuffers`: quem manda corpo
    /// sem anunciar o tipo é um cliente nosso falando o formato nativo.
    pub(crate) fn of_request(content_type: Option<&str>) -> Self {
        match content_type {
            Some(value) if value.to_ascii_lowercase().contains("json") => Self::Json,
            _ => Self::FlatBuffers,
        }
    }

    /// O formato da **resposta**, pelo `Accept`.
    ///
    /// Cabeçalho ausente ou irreconhecível cai em JSON. Um `*/*` — que é o que
    /// um navegador manda — também: entre os dois, o legível serve melhor a
    /// quem não pediu nada.
    pub(crate) fn of_response(accept: Option<&str>) -> Self {
        let Some(accept) = accept.map(str::to_ascii_lowercase) else {
            return Self::Json;
        };

        if accept.contains("json") {
            return Self::Json;
        }

        if accept.contains("flatbuffers") || accept.contains("octet-stream") {
            return Self::FlatBuffers;
        }

        Self::Json
    }

    /// O valor de `Content-Type` a devolver.
    pub(crate) const fn header_value(self) -> &'static str {
        match self {
            Self::Json => JSON,
            Self::FlatBuffers => FLATBUFFERS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn sem_content_type_o_corpo_e_binario() {
        assert_eq!(MediaType::of_request(None), MediaType::FlatBuffers);
        assert_eq!(
            MediaType::of_request(Some(FLATBUFFERS)),
            MediaType::FlatBuffers
        );
        assert_eq!(MediaType::of_request(Some(JSON)), MediaType::Json);
    }

    #[test]
    fn sem_accept_a_resposta_e_legivel() {
        assert_eq!(MediaType::of_response(None), MediaType::Json);
        assert_eq!(MediaType::of_response(Some("*/*")), MediaType::Json);
        assert_eq!(
            MediaType::of_response(Some(FLATBUFFERS)),
            MediaType::FlatBuffers
        );
        // `application/octet-stream` também: é o que um cliente genérico manda
        // quando só sabe que o corpo é binário.
        assert_eq!(
            MediaType::of_response(Some("application/octet-stream")),
            MediaType::FlatBuffers
        );
    }

    #[test]
    fn o_accept_com_qualidade_e_lista_ainda_e_entendido() {
        // Um navegador manda algo como `text/html,application/json;q=0.9,*/*`.
        assert_eq!(
            MediaType::of_response(Some("text/html,application/json;q=0.9,*/*;q=0.8")),
            MediaType::Json
        );
    }
}
