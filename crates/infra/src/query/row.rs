//! A leitura de uma coluna, na forma que a View quer.
//!
//! A hidratação de uma View lê colunas por nome, uma a uma, porque as consultas
//! de leitura projetam expressões (`COUNT(*) AS user_count`, `JSON_ARRAYAGG(…)
//! AS manifest_json`) que não correspondem a struct nenhuma — `FromRow` serve o
//! lado da escrita, onde a linha é a tabela.
//!
//! ## Por que cada função falha em vez de assumir um padrão
//!
//! O PHP daqui era defensivo: coluna ausente virava `0`, enum desconhecido virava
//! a primeira variante. Isso troca um erro por um dado errado — um `density: 0.0`
//! silencioso vira cálculo de peso errado lá na frente, e uma classe de risco
//! escolhida por aproximação afirma que a carga é explosiva quando o banco diz
//! outra coisa. Uma consulta que não sabe ler a própria projeção está quebrada, e
//! é isso que ela deve reportar.
//!
//! A exceção é [`Row::opt_text`], onde a ausência é o dado: uma descrição de
//! telemetria pode ser nula por natureza.

use anyhow::Context;
use sqlx::mysql::MySqlRow;
use sqlx::Row as _;

use crate::entity::codec::Codec;

/// Lê colunas de uma linha do `MariaDB`, com o erro já contextualizado.
///
/// É um namespace, não um valor: oito leitores que só fazem sentido juntos, no
/// molde do `JsonHelper` do PHP. Sem ele, `row::text(...)` e `row::id(...)`
/// seriam funções soltas num módulo cujo nome não diz o que elas têm em comum.
pub(crate) struct Row;

impl Row {
    /// Um texto.
    pub(crate) fn text(row: &MySqlRow, column: &str) -> anyhow::Result<String> {
        row.try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como texto"))
    }

    /// Um texto que pode ser nulo.
    pub(crate) fn opt_text(row: &MySqlRow, column: &str) -> anyhow::Result<Option<String>> {
        row.try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como texto opcional"))
    }

    /// Um inteiro — id cru, contagem, total.
    pub(crate) fn number(row: &MySqlRow, column: &str) -> anyhow::Result<i64> {
        row.try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como inteiro"))
    }

    /// Um real — peso, densidade, capacidade, quantidade.
    pub(crate) fn real(row: &MySqlRow, column: &str) -> anyhow::Result<f64> {
        row.try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como real"))
    }

    /// Um id, já em base62.
    ///
    /// A fronteira `i64` ↔ base62 é desta camada e de mais nenhuma: a View sai com o
    /// id que o cliente vê, e o inteiro do Snowflake não atravessa daqui para cima.
    pub(crate) fn id(row: &MySqlRow, column: &str) -> anyhow::Result<String> {
        Ok(Codec::encode_id(Self::number(row, column)?))
    }

    /// Um id que pode ser nulo — o lado ausente de um `LEFT JOIN`.
    pub(crate) fn opt_id(row: &MySqlRow, column: &str) -> anyhow::Result<Option<String>> {
        let raw: Option<i64> = row
            .try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como inteiro opcional"))?;

        Ok(raw.map(Codec::encode_id))
    }

    /// O índice de um enum, validado contra as variantes que existem.
    ///
    /// A View carrega o índice, não o enum — mas validar aqui é o que impede um
    /// valor que o schema não deveria admitir de sair pelo fio como se fosse
    /// legítimo.
    pub fn enum_index<T>(
        row: &MySqlRow,
        column: &str,
        from: impl Fn(i32) -> Option<T>,
        name: &str,
    ) -> anyhow::Result<i32> {
        let raw = Self::number(row, column)?;
        let index = i32::try_from(raw)
            .with_context(|| format!("coluna `{column}` guarda {raw}, fora da faixa de {name}"))?;

        Codec::decode_enum(index, from, name)?;

        Ok(index)
    }

    /// A coluna `permissions`, que é um array JSON de slugs.
    ///
    /// Um elemento que não seja texto é descartado em vez de derrubar a consulta:
    /// permissão é lista de autorização, e uma entrada corrompida no meio não deve
    /// impedir de ler as outras. Descartar aqui **restringe** o que o papel concede,
    /// que é o lado seguro de errar.
    pub(crate) fn permissions(row: &MySqlRow, column: &str) -> anyhow::Result<Vec<String>> {
        let raw: Option<String> = row
            .try_get(column)
            .with_context(|| format!("coluna `{column}` não veio como JSON"))?;

        let Some(raw) = raw else {
            return Ok(Vec::new());
        };

        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("coluna `{column}` não é JSON válido"))?;

        Ok(parsed
            .as_array()
            .map(|slugs| {
                slugs
                    .iter()
                    .filter_map(|slug| slug.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default())
    }
}
