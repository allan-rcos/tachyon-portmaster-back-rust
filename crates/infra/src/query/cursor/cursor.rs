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
    pub(crate) fn encode(last_id: i64, filters: &CursorFilters) -> String {
        let payload = Payload {
            id: last_id,
            f: filters.clone(),
        };

        // A serialização de um `BTreeMap<String, String>` com um `i64` não tem
        // caminho de falha; se um dia tivesse, um token vazio faria a paginação
        // recomeçar, que é degradação aceitável para uma consulta de leitura.
        let json = serde_json::to_vec(&payload).unwrap_or_default();

        URL_SAFE_NO_PAD.encode(json)
    }

    /// Lê um token, ou devolve `None` para recomeçar do princípio.
    ///
    /// Toda recusa é o mesmo `None`: token ausente, token vazio, texto que não é
    /// base64url, JSON que não tem a forma esperada, e token emitido sob outros
    /// filtros. Quem chama trata todos como "primeira página", e é por isso que
    /// não existe erro de cursor.
    pub(crate) fn decode(token: Option<&str>, current_filters: &CursorFilters) -> Option<Self> {
        let token = token.filter(|t| !t.is_empty())?;

        let json = URL_SAFE_NO_PAD.decode(token).ok()?;
        let payload: Payload = serde_json::from_slice(&json).ok()?;

        // Filtros mudaram desde a emissão: o token descreve uma varredura que
        // não é mais a desta requisição.
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn busca(termo: &str) -> CursorFilters {
        CursorFilters::of([("limit", "20".to_owned()), ("search", termo.to_owned())])
    }

    #[test]
    fn ida_e_volta_preserva_o_id() {
        let f = busca("cimento");
        let token = Cursor::encode(9_876_543_210, &f);

        assert_eq!(
            Cursor::decode(Some(&token), &f),
            Some(Cursor {
                last_id: 9_876_543_210
            })
        );
    }

    #[test]
    fn o_token_atravessa_uma_querystring_intacto() {
        // Base64url sem padding: nada de `+`, `/` ou `=`, que precisariam de
        // escape e voltariam corrompidos.
        let token = Cursor::encode(i64::MAX, &busca("álcool etílico 70%"));

        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "token com caractere que a querystring alteraria: {token}"
        );
    }

    #[test]
    fn cursor_de_outro_filtro_e_ignorado() {
        // O caso que motiva guardar os filtros: o cliente trocou a busca e
        // reenviou o cursor antigo. Continuar dali entregaria uma página do
        // conjunto anterior.
        let token = Cursor::encode(100, &busca("cimento"));

        assert_eq!(Cursor::decode(Some(&token), &busca("areia")), None);
    }

    #[test]
    fn toda_recusa_e_a_mesma_ausencia() {
        let f = busca("cimento");

        assert_eq!(Cursor::decode(None, &f), None, "sem token");
        assert_eq!(Cursor::decode(Some(""), &f), None, "token vazio");
        assert_eq!(
            Cursor::decode(Some("!!não é base64!!"), &f),
            None,
            "não é base64url"
        );
        assert_eq!(
            Cursor::decode(Some(&URL_SAFE_NO_PAD.encode("nem json")), &f),
            None,
            "não é o JSON esperado"
        );
    }

    #[test]
    fn sem_cursor_a_varredura_comeca_do_zero() {
        assert_eq!(Cursor::last_id_or_start(None, &busca("cimento")), 0);
    }

    #[test]
    fn pagina_incompleta_nao_emite_proximo_cursor() {
        // Veio menos que o limite, então acabou. Emitir cursor aqui custaria ao
        // cliente uma requisição a mais para descobrir que não há nada.
        assert_eq!(Cursor::next(7, 20, 500, &busca("cimento")), None);
    }

    #[test]
    fn pagina_cheia_emite_o_proximo_cursor() {
        let f = busca("cimento");
        let token = Cursor::next(20, 20, 500, &f).expect("página cheia deveria emitir cursor");

        assert_eq!(
            Cursor::decode(Some(&token), &f),
            Some(Cursor { last_id: 500 })
        );
    }

    #[test]
    fn a_ordem_dos_filtros_nao_muda_o_token() {
        // Dois mapas com os mesmos pares têm que gerar o mesmo token, senão um
        // cursor válido seria recusado por acidente de iteração.
        let a = CursorFilters::of([("limit", "20".to_owned()), ("search", "cal".to_owned())]);
        let b = CursorFilters::of([("search", "cal".to_owned()), ("limit", "20".to_owned())]);

        assert_eq!(Cursor::encode(42, &a), Cursor::encode(42, &b));
    }
}
