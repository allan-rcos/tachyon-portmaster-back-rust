//! O cursor de paginação keyset.

use crate::query::cursor::CursorFilters;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// O conteúdo do token, como ele viaja.
#[derive(Serialize, Deserialize)]
struct Payload {
    /// Id da última linha da página servida.
    id: i64,
    /// Os filtros sob os quais o token vale.
    f: CursorFilters,
}

/// Um cursor keyset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    /// A página seguinte começa estritamente depois deste id.
    pub(crate) last_id: i64,
}

impl Cursor {
    /// Emite o token da próxima página.
    /// Emite o token da próxima página.
    ///
    /// A serialização de um `BTreeMap<String, String>` com um `i64` não tem
    /// caminho de falha; se um dia tivesse, um token vazio faria a paginação
    /// recomeçar, que é degradação aceitável para uma consulta de leitura.
    pub(crate) fn encode(last_id: i64, filters: &CursorFilters) -> String {
        let payload = Payload {
            id: last_id,
            f: filters.clone(),
        };

        let json = serde_json::to_vec(&payload).unwrap_or_default();

        URL_SAFE_NO_PAD.encode(json)
    }

    /// Lê um token, ou devolve `None` para recomeçar do princípio.
    ///
    /// Toda recusa é o mesmo `None`: token ausente, token vazio, texto que não é
    /// base64url, JSON que não tem a forma esperada, e token emitido sob outros
    /// filtros. Quem chama trata todos como "primeira página", e é por isso que
    /// não existe erro de cursor.
    /// Lê o token, se ele ainda descreve esta consulta.
    ///
    /// Filtros diferentes dos da emissão devolvem `None`: o token descreve uma
    /// varredura que não é mais a desta requisição, e continuar dali entregaria
    /// uma página do conjunto anterior.
    pub(crate) fn decode(token: Option<&str>, current_filters: &CursorFilters) -> Option<Self> {
        let token = token.filter(|t| !t.is_empty())?;

        let json = URL_SAFE_NO_PAD.decode(token).ok()?;
        let payload: Payload = serde_json::from_slice(&json).ok()?;

        if &payload.f != current_filters {
            return None;
        }

        Some(Self {
            last_id: payload.id,
        })
    }

    /// O id de onde partir, com `0` quando não há cursor válido.
    ///
    /// `0` funciona como origem porque todo id é um Snowflake positivo.
    pub(crate) fn last_id_or_start(token: Option<&str>, current_filters: &CursorFilters) -> i64 {
        Self::decode(token, current_filters).map_or(0, |cursor| cursor.last_id)
    }

    /// O token da próxima página — ou `None` quando esta foi a última.
    ///
    /// Só emite quando a página veio **cheia**: uma página menor que o limite
    /// significa que a varredura acabou, e devolver cursor ali levaria o cliente
    /// a uma requisição a mais só para receber nada.
    pub(crate) fn next(
        served: usize,
        limit: u32,
        last_id: i64,
        filters: &CursorFilters,
    ) -> Option<String> {
        (served == limit as usize && last_id > 0).then(|| Self::encode(last_id, filters))
    }
}
