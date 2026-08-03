//! As *strategies*: sabem um formato, e nada sobre o payload.
//!
//! Esta é metade da separação que sustenta o wire. Uma strategy conhece **como**
//! escrever ou ler um formato e é indiferente ao **quê** — todo tipo de
//! FlatBuffers se serializa igual, e todo tipo serde também. A outra metade são
//! as [factories](super::factory), que conhecem os dados e nada sobre o formato.
//!
//! Manter as duas separadas é o que faz um terceiro formato custar uma strategy
//! nova, e não uma varredura por todos os handlers.
//!
//! ## Um conjunto de tipos, dois formatos
//!
//! O `planus` gera os tipos das tabelas `.fbs` **com os derives do serde**. A
//! mesma struct, portanto, escreve JSON pelo serde e FlatBuffers pelo builder —
//! e o JSON que ela produz é exatamente o que `swagger/swagger.json` documenta,
//! incluindo os enums como nome (`"Class3FlammableLiquids"`). Um segundo
//! conjunto de tipos escrito à mão seria duplicação que só teria como divergir.
//!
//! ## Os padrões da negociação não são simétricos
//!
//! Sem `Content-Type`, o corpo é lido como **FlatBuffers**; sem `Accept`, a
//! resposta sai em **JSON**. Assimetria herdada do PHP, e sensata: quem manda
//! corpo sem anunciar o tipo é um cliente nosso falando o formato nativo, e quem
//! não pede formato nenhum costuma ser um humano com um `curl`.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ApiError;

/// `application/json`.
pub(crate) const JSON: &str = "application/json";

/// O tipo nativo desta API.
pub(crate) const FLATBUFFERS: &str = "application/x-flatbuffers";

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
    /// Cabeçalho ausente ou irreconhecível cai em FlatBuffers.
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
    pub(crate) fn header_value(self) -> &'static str {
        match self {
            Self::Json => JSON,
            Self::FlatBuffers => FLATBUFFERS,
        }
    }
}

/// Uma tabela que pode chegar como corpo de requisição.
///
/// O `DeserializeOwned` cobre o JSON; o [`from_flatbuffers`](Self::from_flatbuffers)
/// cobre o binário. Implementado pela macro [`wire_request!`](crate::wire_request)
/// para cada tabela de request, porque cada uma tem o seu tipo `Ref`.
pub(crate) trait WireRequest: Sized + DeserializeOwned {
    /// Lê a tabela de um buffer FlatBuffers.
    fn from_flatbuffers(bytes: &[u8]) -> Result<Self, ApiError>;
}

/// Uma tabela que pode sair como corpo de resposta.
///
/// Não tem método nenhum: é só o par de capacidades que as duas strategies
/// exigem. Toda tabela gerada pelo planus a satisfaz automaticamente.
pub(crate) trait WireResponse: Serialize + planus::WriteAsOffset<Self> {}

impl<T> WireResponse for T where T: Serialize + planus::WriteAsOffset<T> {}

/// Lê um corpo, sem saber que tabela é.
pub(crate) trait DecodeStrategy {
    /// Desserializa o corpo na tabela pedida.
    fn decode<R: WireRequest>(&self, bytes: &[u8]) -> Result<R, ApiError>;
}

/// Escreve um corpo, sem saber que tabela é.
pub(crate) trait EncodeStrategy {
    /// Serializa a tabela no formato desta strategy.
    fn encode<R: WireResponse>(&self, value: &R) -> Result<Vec<u8>, ApiError>;
}

/// JSON, pelo serde.
pub(crate) struct JsonWire;

impl DecodeStrategy for JsonWire {
    fn decode<R: WireRequest>(&self, bytes: &[u8]) -> Result<R, ApiError> {
        // A mensagem do serde nomeia o campo e a linha — é erro do cliente, e
        // devolvê-la ajuda quem está integrando. Não há segredo nela: o corpo é
        // o que ele mesmo mandou.
        serde_json::from_slice(bytes)
            .map_err(|e| ApiError::malformed_body(format!("corpo JSON inválido: {e}")))
    }
}

impl EncodeStrategy for JsonWire {
    fn encode<R: WireResponse>(&self, value: &R) -> Result<Vec<u8>, ApiError> {
        serde_json::to_vec(value).map_err(|e| {
            ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
    }
}

/// FlatBuffers, pelo planus.
pub(crate) struct FlatBuffersWire;

impl DecodeStrategy for FlatBuffersWire {
    fn decode<R: WireRequest>(&self, bytes: &[u8]) -> Result<R, ApiError> {
        R::from_flatbuffers(bytes)
    }
}

impl EncodeStrategy for FlatBuffersWire {
    fn encode<R: WireResponse>(&self, value: &R) -> Result<Vec<u8>, ApiError> {
        // O builder é descartado com o buffer: reaproveitá-lo entre requisições
        // exigiria sincronizá-lo, e ele existe para durar uma serialização.
        let mut builder = planus::Builder::new();

        Ok(builder.finish(value, None).to_vec())
    }
}

/// Lê o corpo no formato negociado.
///
/// O `match` fica aqui, e não num `dyn`: [`DecodeStrategy::decode`] é genérico
/// e por isso a trait não é object-safe — o que é conveniente, porque mantém o
/// caminho de request inteiramente estático.
pub(crate) fn decode<R: WireRequest>(media: MediaType, bytes: &[u8]) -> Result<R, ApiError> {
    match media {
        MediaType::Json => JsonWire.decode(bytes),
        MediaType::FlatBuffers => FlatBuffersWire.decode(bytes),
    }
}

/// Escreve o corpo no formato negociado.
pub(crate) fn encode<R: WireResponse>(media: MediaType, value: &R) -> Result<Vec<u8>, ApiError> {
    match media {
        MediaType::Json => JsonWire.encode(value),
        MediaType::FlatBuffers => FlatBuffersWire.encode(value),
    }
}

/// Declara uma tabela gerada como corpo de requisição aceitável.
///
/// Existe porque cada tabela tem o seu tipo `Ref` — o planus não expõe essa
/// associação por trait, então a ligação é escrita uma vez por tipo aqui em vez
/// de repetida em cada handler.
#[macro_export]
macro_rules! wire_request {
    ($owned:path, $reference:path) => {
        impl $crate::wire::negotiate::WireRequest for $owned {
            fn from_flatbuffers(bytes: &[u8]) -> Result<Self, $crate::error::ApiError> {
                use ::planus::ReadAsRoot as _;

                let reference = <$reference>::read_as_root(bytes).map_err(|e| {
                    $crate::error::ApiError::malformed_body(format!(
                        "corpo FlatBuffers inválido: {e}"
                    ))
                })?;

                ::core::convert::TryInto::try_into(reference).map_err(|e| {
                    $crate::error::ApiError::malformed_body(format!(
                        "corpo FlatBuffers incompleto: {e}"
                    ))
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::tables as fbs;
    use pretty_assertions::assert_eq;

    #[test]
    fn sem_content_type_o_corpo_e_binario() {
        // Quem manda corpo sem anunciar o tipo é um cliente nosso falando o
        // formato nativo.
        assert_eq!(MediaType::of_request(None), MediaType::FlatBuffers);
        assert_eq!(
            MediaType::of_request(Some(FLATBUFFERS)),
            MediaType::FlatBuffers
        );
        assert_eq!(MediaType::of_request(Some(JSON)), MediaType::Json);
    }

    #[test]
    fn sem_accept_a_resposta_e_legivel() {
        // Quem não pediu formato costuma ser um humano com um curl.
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

    #[test]
    fn a_mesma_tabela_sai_nos_dois_formatos() {
        // É o ponto do desenho: um conjunto de tipos, duas strategies.
        let response = fbs::product::ProductResponse {
            id: Some("aZ3".into()),
            name: Some("Cimento".into()),
            density: 1.44,
            risk_class: fbs::common::RiskClass::Class3FlammableLiquids,
        };

        let json = encode(MediaType::Json, &response).unwrap();
        let binary = encode(MediaType::FlatBuffers, &response).unwrap();

        assert_eq!(
            String::from_utf8(json).unwrap(),
            r#"{"id":"aZ3","name":"Cimento","density":1.44,"risk_class":"Class3FlammableLiquids"}"#,
            "o JSON precisa bater com o que swagger.json documenta"
        );
        assert!(!binary.is_empty());
    }

    #[test]
    fn o_corpo_binario_volta_a_ser_a_mesma_tabela() {
        let sent = fbs::auth::LoginRequest {
            email: "ana@portmaster.local".into(),
            password: "Portmaster1".into(),
        };

        let bytes = {
            let mut builder = planus::Builder::new();
            builder.finish(&sent, None).to_vec()
        };

        let received: fbs::auth::LoginRequest = decode(MediaType::FlatBuffers, &bytes).unwrap();

        assert_eq!(received, sent);
    }

    #[test]
    fn o_corpo_json_volta_a_ser_a_mesma_tabela() {
        let received: fbs::auth::LoginRequest = decode(
            MediaType::Json,
            br#"{"email":"ana@portmaster.local","password":"Portmaster1"}"#,
        )
        .unwrap();

        assert_eq!(received.email, "ana@portmaster.local");
    }

    #[test]
    fn corpo_ilegivel_vira_400_e_nao_panico() {
        // Um cliente que manda lixo não deve derrubar o handler.
        let json: Result<fbs::auth::LoginRequest, _> = decode(MediaType::Json, b"{{{");
        let binary: Result<fbs::auth::LoginRequest, _> =
            decode(MediaType::FlatBuffers, b"\x00\x01");

        assert_eq!(
            json.err().map(|e| e.status()),
            Some(axum::http::StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            binary.err().map(|e| e.status()),
            Some(axum::http::StatusCode::BAD_REQUEST)
        );
    }
}
