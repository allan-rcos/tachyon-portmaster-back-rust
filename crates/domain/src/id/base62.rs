//! A fronteira entre o id que o banco guarda e o id que o mundo vê.
//!
//! Dentro da `infra` um id de entidade é um `BIGINT`; em qualquer outro lugar é
//! esta string. Duas razões:
//!
//! * **JavaScript.** Um `i64` não sobrevive a um `Number`: acima de 2^53 o
//!   cliente silenciosamente arredonda o id e passa a falar de outra linha. Uma
//!   string atravessa intacta.
//! * **Trocar o gerador.** Como o resto do sistema trata id como string opaca,
//!   migrar de Snowflake para ULID ou UUID um dia mexe só na `infra`.
//!
//! O alfabeto é `0-9A-Za-z` nessa ordem — a mesma do PHP, e a que faz a rota
//! casar `[A-Za-z0-9]+`.

/// Dígitos na ordem que define o valor de cada caractere.
const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Falha ao decodificar uma string base62.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Base62Error {
    /// String vazia não representa número nenhum.
    #[error("base62 não decodifica string vazia")]
    Empty,

    /// Caractere fora do alfabeto.
    #[error("caractere base62 inválido: {0:?}")]
    InvalidCharacter(char),

    /// Valor grande demais para caber num id.
    ///
    /// Nenhum id emitido por esta aplicação chega aqui — um Snowflake estourar
    /// `i64` é problema de 2093. Um valor deste tamanho veio da URL, e é tão
    /// inválido quanto um caractere fora do alfabeto.
    #[error("valor base62 fora da faixa: {0}")]
    OutOfRange(String),
}

/// Compacta um inteiro não-negativo em base62.
pub fn encode(mut number: i64) -> String {
    debug_assert!(number >= 0, "base62 codifica apenas inteiros não-negativos");
    if number <= 0 {
        return "0".to_string();
    }

    let mut buffer = Vec::with_capacity(11); // 11 dígitos cobrem todo o i64
    while number > 0 {
        let digit = (number % 62) as usize;
        buffer.push(ALPHABET[digit]);
        number /= 62;
    }
    buffer.reverse();

    // Todo byte veio de ALPHABET, que é ASCII.
    String::from_utf8(buffer).unwrap_or_default()
}

/// Recupera o inteiro por trás de uma string base62.
pub fn decode(value: &str) -> Result<i64, Base62Error> {
    if value.is_empty() {
        return Err(Base62Error::Empty);
    }

    let mut number: i64 = 0;
    for character in value.chars() {
        let position = ALPHABET
            .iter()
            .position(|&c| c as char == character)
            .ok_or(Base62Error::InvalidCharacter(character))? as i64;

        // Checa antes de multiplicar: depois do overflow não há como saber que
        // ele ocorreu.
        number = number
            .checked_mul(62)
            .and_then(|n| n.checked_add(position))
            .ok_or_else(|| Base62Error::OutOfRange(value.to_string()))?;
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn codifica_os_casos_de_borda() {
        assert_eq!(encode(0), "0");
        assert_eq!(encode(1), "1");
        assert_eq!(encode(61), "z");
        assert_eq!(encode(62), "10");
    }

    #[test]
    fn ida_e_volta_preserva_o_numero() {
        for value in [0_i64, 1, 61, 62, 3843, 1_234_567_890, i64::MAX] {
            assert_eq!(decode(&encode(value)), Ok(value), "falhou para {value}");
        }
    }

    #[test]
    fn recusa_entrada_invalida() {
        assert_eq!(decode(""), Err(Base62Error::Empty));
        assert_eq!(decode("abc-def"), Err(Base62Error::InvalidCharacter('-')));
        assert_eq!(decode("çé"), Err(Base62Error::InvalidCharacter('ç')));
    }

    #[test]
    fn recusa_valor_alem_do_i64() {
        // Doze dígitos 'z' passam de i64::MAX. Isto não é um id nosso: é uma URL
        // inventada, e precisa falhar como tal em vez de estourar.
        let overflow = "z".repeat(12);
        assert_eq!(
            decode(&overflow),
            Err(Base62Error::OutOfRange(overflow.clone()))
        );
    }
}
