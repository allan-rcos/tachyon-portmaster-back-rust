//! Normalização de texto para busca.
//!
//! A coluna `search_*` existe porque o banco pode trabalhar a favor: em vez de
//! forçar um `LIKE` sobre a string original — que erraria por acento, caixa ou
//! espaço duplicado —, guarda-se uma versão normalizada ao lado, indexada, e a
//! busca roda sobre ela.
//!
//! O objeto de domínio segue fiel ao que o usuário digitou; quem ganha uma
//! coluna auxiliar é a tabela, e só a `infra` sabe que ela existe.

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization as _;

/// Normaliza um texto para a coluna de busca.
///
/// Continua sendo uma função só, e não a expressão repetida em cada ponto de
/// uso, porque os dois lados **têm** de produzir o mesmo byte: quem grava a
/// coluna e quem monta o `LIKE` que a consulta. Duas cópias que divergissem não
/// dariam erro nenhum — só devolveriam busca vazia para registro que existe.
pub(crate) struct SearchKey;

impl SearchKey {
    /// Reduz um texto à chave usada pelo filtro `LIKE`.
    ///
    /// Tira acento, baixa a caixa e colapsa espaço. A decomposição é a do
    /// Unicode (NFKD), não uma tabela de letras latinas escrita à mão: `Á` vira
    /// `A` seguido de um acento combinante, e é o acento que se descarta.
    ///
    /// O que **não** tem decomposição é preservado, e é por isso que a
    /// normalização é essa e não uma transliteração para ASCII: `Ячмень` fica
    /// em cirílico, minúsculo. Reduzi-lo a `yachmen` tornaria o registro
    /// inencontrável para quem o digitou como ele é.
    pub(crate) fn of(value: &str) -> String {
        let folded: String = value
            .nfkd()
            .filter(|character| !is_combining_mark(*character))
            .flat_map(char::to_lowercase)
            .collect();

        folded.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
#[path = "tests/search_key_test.rs"]
mod tests;
