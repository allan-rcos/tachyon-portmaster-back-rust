//! A leitura de campos de um objeto JSON.
//!
//! É o `JsonHelper` do PHP: um namespace de leitores que uma factory usa para
//! tirar cada campo do `Map` sem repetir a mesma dança de `get` + `as_str` +
//! `to_owned` em cada uma das 15 mensagens de requisição.
//!
//! ## Ausente e inválido são a mesma coisa aqui
//!
//! Todo leitor devolve `Option`, e um campo com o tipo errado vira `None` em vez
//! de erro. Não é desleixo: quem recusa é o `TableModule`, que sabe **quais**
//! campos são obrigatórios e devolve todos os problemas de uma vez. Um erro
//! levantado aqui interromperia na primeira falha e devolveria um por vez.

use serde_json::{Map, Value};

/// Lê campos de um objeto JSON.
// Os leitores que ainda não têm chamador são os das mensagens que as fatias
// 4.2–4.4 vão migrar. Sai quando a última rota estiver no wire novo.
pub(crate) struct Json;

// Os leitores sem chamador são os das mensagens que as fatias 4.2–4.4 vão
// migrar. O `allow` sai quando a última rota estiver no wire novo.
#[allow(
    dead_code,
    reason = "leitores usados pelas mensagens ainda não migradas (fatias 4.2-4.4)"
)]
impl Json {
    /// Um texto.
    pub(crate) fn text(source: &Map<String, Value>, field: &str) -> Option<String> {
        source.get(field).and_then(Value::as_str).map(str::to_owned)
    }

    /// Um inteiro de 64 bits.
    pub(crate) fn number(source: &Map<String, Value>, field: &str) -> Option<i64> {
        source.get(field).and_then(Value::as_i64)
    }

    /// Um inteiro de 32 bits, saturado na faixa.
    pub(crate) fn int(source: &Map<String, Value>, field: &str) -> Option<i32> {
        Self::number(source, field).map(|value| {
            i32::try_from(value).unwrap_or_else(|_| {
                if value.is_negative() {
                    i32::MIN
                } else {
                    i32::MAX
                }
            })
        })
    }

    /// Um número com casas decimais.
    pub(crate) fn real(source: &Map<String, Value>, field: &str) -> Option<f64> {
        source.get(field).and_then(Value::as_f64)
    }

    /// Um booleano.
    pub(crate) fn flag(source: &Map<String, Value>, field: &str) -> Option<bool> {
        source.get(field).and_then(Value::as_bool)
    }

    /// Uma lista de textos.
    ///
    /// Entradas que não são texto são descartadas — a lista existe para ser
    /// consumida inteira, e um elemento inválido no meio não deve derrubar os
    /// demais.
    pub(crate) fn texts(source: &Map<String, Value>, field: &str) -> Option<Vec<String>> {
        source.get(field).and_then(Value::as_array).map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn source() -> Map<String, Value> {
        match serde_json::json!({
            "name": "Cimento",
            "density": 1.44,
            "count": 7,
            "active": true,
            "roles": ["a", 2, "b"],
        }) {
            Value::Object(map) => map,
            _ => unreachable!("o literal acima é um objeto"),
        }
    }

    #[test]
    fn le_cada_tipo_no_seu_leitor() {
        let s = source();

        assert_eq!(Json::text(&s, "name"), Some("Cimento".to_owned()));
        assert_eq!(Json::real(&s, "density"), Some(1.44));
        assert_eq!(Json::number(&s, "count"), Some(7));
        assert_eq!(Json::flag(&s, "active"), Some(true));
    }

    #[test]
    fn campo_ausente_ou_do_tipo_errado_e_none() {
        let s = source();

        // Ausente e presente-com-tipo-errado são o mesmo `None`: quem decide se
        // isso é um problema é o TableModule, que sabe o que é obrigatório.
        assert_eq!(Json::text(&s, "inexistente"), None);
        assert_eq!(Json::text(&s, "density"), None);
        assert_eq!(Json::number(&s, "name"), None);
    }

    #[test]
    fn a_lista_descarta_o_que_nao_e_texto() {
        assert_eq!(
            Json::texts(&source(), "roles"),
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
    }
}
