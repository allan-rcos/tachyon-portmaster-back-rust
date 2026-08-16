//! Qual dos dois formatos está em jogo.

/// `application/json`.
const JSON: &str = "application/json";

/// O tipo nativo desta API.
const FLATBUFFERS: &str = "application/x-flatbuffers";

/// O que um cliente genérico manda quando só sabe que o corpo não é texto.
const OCTET_STREAM: &str = "application/octet-stream";

/// O curinga que um navegador manda em toda navegação.
const ANY: &str = "*/*";

/// Qual dos dois formatos está em jogo.
///
/// Os dois construtores **falham** quando o cabeçalho pede um tipo que não
/// sabemos produzir nem ler. Cair num padrão em silêncio daria a quem pediu XML
/// um JSON com `200` — o pior dos dois mundos, porque nem recebe o que pediu nem
/// descobre que não vai receber.
/// Quem traduz a falha em status é o middleware: `406` na saída, `415` na
/// entrada.
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
    /// Cabeçalho ausente cai em `FlatBuffers`: quem manda corpo sem anunciar o
    /// tipo é um cliente nosso falando o formato nativo. Cabeçalho presente e
    /// desconhecido é erro — ele anunciou alguma coisa, e não foi nenhuma das
    /// duas que lemos.
    pub(crate) fn of_request(content_type: Option<&str>) -> anyhow::Result<Self> {
        let Some(content_type) = content_type.map(str::to_ascii_lowercase) else {
            return Ok(Self::FlatBuffers);
        };

        Self::of_header(&content_type).ok_or_else(|| {
            anyhow::anyhow!(
                "não sabemos ler um corpo em `{content_type}`: use {JSON} ou {FLATBUFFERS}"
            )
        })
    }

    /// O formato da **resposta**, pelo `Accept`.
    ///
    /// Cabeçalho ausente cai em JSON, e um `*/*` em qualquer posição também —
    /// entre os dois, o legível serve melhor a quem não pediu nada em
    /// particular, e é o que um navegador manda. Só um tipo concreto que não
    /// sabemos escrever é erro; tratar o curinga como erro daria `406` para todo
    /// navegador que abrisse a API.
    pub(crate) fn of_response(accept: Option<&str>) -> anyhow::Result<Self> {
        let Some(accept) = accept.map(str::to_ascii_lowercase) else {
            return Ok(Self::Json);
        };

        if accept.contains(ANY) {
            return Ok(Self::Json);
        }

        Self::of_header(&accept).ok_or_else(|| {
            anyhow::anyhow!(
                "não sabemos escrever uma resposta em `{accept}`: use {JSON} ou {FLATBUFFERS}"
            )
        })
    }

    /// O valor de `Content-Type` a devolver.
    pub(crate) const fn header_value(self) -> &'static str {
        match self {
            Self::Json => JSON,
            Self::FlatBuffers => FLATBUFFERS,
        }
    }

    /// O formato que um cabeçalho já em minúsculas nomeia.
    ///
    /// Busca por substring, e não por igualdade, porque os dois cabeçalhos vêm
    /// com parâmetros e listas: `application/json; charset=utf-8` de um lado,
    /// `text/html,application/json;q=0.9` do outro.
    fn of_header(header: &str) -> Option<Self> {
        if header.contains("json") {
            return Some(Self::Json);
        }

        if header.contains("flatbuffers") || header.contains(OCTET_STREAM) {
            return Some(Self::FlatBuffers);
        }

        None
    }
}
