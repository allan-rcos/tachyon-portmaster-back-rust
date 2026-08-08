//! Normalização de texto para busca.
//!
//! A coluna `search_*` existe porque o banco pode trabalhar a favor: em vez de
//! forçar um `LIKE` sobre a string original — que erraria por acento, caixa ou
//! espaço duplicado —, guarda-se uma versão normalizada ao lado, indexada, e a
//! busca roda sobre ela.
//!
//! O objeto de domínio segue fiel ao que o usuário digitou; quem ganha uma
//! coluna auxiliar é a tabela, e só a `infra` sabe que ela existe.

/// Normaliza um texto para a coluna de busca.
///
/// Namespace de uma função só, pelo mesmo motivo do PHP: o nome do tipo diz o
/// que a função produz, e `SearchKey::of(...)` lê melhor que `text::search_key(...)`.
pub(crate) struct SearchKey;

impl SearchKey {
    /// Reduz um texto à chave usada pelo filtro `LIKE`.
    ///
    /// Tira acento, baixa a caixa e colapsa espaço. É deliberadamente ingênuo sobre
    /// alfabetos não-latinos: o que não tem decomposição conhecida passa intacto, o
    /// que é melhor do que descartar o caractere e tornar o registro inencontrável.
    pub(crate) fn of(value: &str) -> String {
        let mut normalized = String::with_capacity(value.len());
        let mut last_was_space = true; // começa `true` para comer espaço à esquerda

        for character in value.chars() {
            if character.is_whitespace() {
                // Colapsa qualquer corrida de espaço num só.
                if !last_was_space {
                    normalized.push(' ');
                    last_was_space = true;
                }
                continue;
            }

            last_was_space = false;
            match Self::deaccent(character) {
                Some(base) => normalized.push_str(base),
                // Sem decomposição conhecida: preserva o caractere em vez de
                // descartá-lo. Descartar tornaria inencontrável qualquer nome fora
                // do alfabeto latino.
                None => normalized.extend(character.to_lowercase()),
            }
        }

        // Come o espaço à direita, se sobrou um.
        if normalized.ends_with(' ') {
            normalized.pop();
        }

        normalized
    }

    /// Troca uma letra acentuada pela sua base latina, se conhecer a decomposição.
    ///
    /// Cobre o que aparece em português e nas línguas latinas vizinhas. `None`
    /// significa "não sei decompor", e o chamador preserva o caractere original.
    const fn deaccent(character: char) -> Option<&'static str> {
        let base = match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' | 'Å' => {
                "a"
            }
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => "e",
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => "i",
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => "o",
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => "u",
            'ç' | 'Ç' => "c",
            'ñ' | 'Ñ' => "n",
            'ý' | 'ÿ' | 'Ý' => "y",
            _ => return None,
        };

        Some(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn key(value: &str) -> String {
        SearchKey::of(value)
    }

    #[test]
    fn baixa_a_caixa() {
        assert_eq!(key("Soja Tipo 2"), "soja tipo 2");
    }

    #[test]
    fn colapsa_espaco() {
        assert_eq!(key("  soja   tipo  2  "), "soja tipo 2");
        assert_eq!(key("soja\ttipo\n2"), "soja tipo 2");
    }

    #[test]
    fn tira_acento() {
        // É o ponto da coluna auxiliar: quem busca "acucar" precisa achar
        // "Açúcar".
        assert_eq!(key("Açúcar"), "acucar");
        assert_eq!(key("Óleo de Soja"), "oleo de soja");
        assert_eq!(key("PIÑA"), "pina");
    }

    #[test]
    fn texto_vazio_vira_chave_vazia() {
        assert_eq!(key(""), "");
        assert_eq!(key("   "), "");
    }

    #[test]
    fn preserva_alfabeto_que_nao_sabe_decompor() {
        // Descartar o que não tem decomposição conhecida tornaria o registro
        // inencontrável — pior do que deixá-lo passar sem normalizar.
        assert_eq!(key("Ячмень"), "ячмень");
        assert_eq!(key("大豆 A"), "大豆 a");
    }
}
