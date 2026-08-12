//! O id de uma linha, nas duas formas em que ele existe.

use std::sync::OnceLock;

use anyhow::{anyhow, Context as _};

use crate::entity::codec::Codec;

/// O id de uma linha: `BIGINT` na coluna, base62 no fio.
///
/// Um campo só onde antes havia dois. Cada entity guardava `id: String` e
/// `raw_id: i64` lado a lado, e os dois precisavam concordar — mantidos em
/// sincronia à mão no `from_row` e no `from_domain`, sem nada que impedisse um
/// de ficar para trás.
///
/// ## O base62 é derivado, e só quando alguém o pede
///
/// A coluna guarda o `BIGINT`; o base62 existe porque o trait de domínio expõe
/// `fn id(&self) -> &str`. Como aquela assinatura devolve **referência**, não dá
/// para codificar na hora e devolver o resultado — daí o [`OnceLock`], que
/// codifica na primeira leitura e guarda o resultado para as seguintes.
///
/// Quem lê uma linha só para pegar a FK, ou para conferir que ela existe, nunca
/// paga a codificação.
#[derive(Debug, Clone)]
pub(crate) struct EntityId {
    /// O id como a coluna o guarda.
    raw: i64,
    /// O base62 correspondente, vazio até alguém pedir.
    encoded: OnceLock<String>,
}

impl EntityId {
    /// O id como o banco o guarda.
    pub(crate) const fn raw(&self) -> i64 {
        self.raw
    }

    /// O id como o domínio o expõe.
    ///
    /// A primeira chamada codifica; as seguintes devolvem o que ficou guardado.
    pub(crate) fn as_str(&self) -> &str {
        self.encoded.get_or_init(|| Codec::encode_id(self.raw))
    }
}

impl TryFrom<i64> for EntityId {
    type Error = anyhow::Error;

    /// Adota o id que veio da coluna.
    ///
    /// Recusa negativo em vez de codificá-lo: o Snowflake não acende o bit de
    /// sinal, então um id negativo veio de uma linha que o schema não deveria
    /// admitir. Antes o `encode_id` o transformava em `0` — afirmava uma
    /// identidade que a linha não tem, que é a mesma coisa que escolher variante
    /// de enum por aproximação.
    fn try_from(raw: i64) -> Result<Self, Self::Error> {
        if raw.is_negative() {
            return Err(anyhow!("id negativo na coluna: {raw}"));
        }

        Ok(Self {
            raw,
            encoded: OnceLock::new(),
        })
    }
}

impl TryFrom<&str> for EntityId {
    type Error = anyhow::Error;

    /// Adota o id que veio do domínio, que já está em base62.
    ///
    /// Popula os dois lados de uma vez: quem chega por aqui veio de um
    /// `from_domain`, já tem a string em mãos, e guardá-la evita recodificar o
    /// que acabou de ser decodificado.
    fn try_from(id: &str) -> Result<Self, Self::Error> {
        let raw = Codec::decode_id(id).with_context(|| format!("id de domínio inválido: {id}"))?;
        let encoded = OnceLock::new();

        let _ = encoded.set(id.to_owned());

        Ok(Self { raw, encoded })
    }
}

#[cfg(test)]
#[path = "tests/entity_id_test.rs"]
mod tests;
