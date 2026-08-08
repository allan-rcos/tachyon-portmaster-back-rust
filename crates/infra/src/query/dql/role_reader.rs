//! A leitura de um `RoleViewItem` a partir de uma linha.
//!
//! Namespace porque as duas leituras são a mesma projeção vista de dois
//! ângulos: a consulta de papel lê as colunas direto, e a de conta as lê
//! prefixadas, porque `id` e `name` existem dos dois lados do `JOIN`. Duplicar
//! a lista de colunas nos dois lugares seria a chance de uma divergir.

use crate::query::row::Row;
use crate::query::views::RoleViewItem;
use sqlx::mysql::MySqlRow;

/// Lê um papel de uma linha, com ou sem prefixo de `JOIN`.
pub(crate) struct RoleReader;

impl RoleReader {
    /// Uma linha de `roles` como a View a quer.
    pub(crate) fn item(row: &MySqlRow) -> anyhow::Result<RoleViewItem> {
        Self::item_prefixed(row, "")
    }

    /// A mesma leitura, quando as colunas do papel vêm prefixadas.
    ///
    /// Nas consultas de conta o papel chega aninhado no usuário, e as duas metades
    /// da linha teriam `id` e `name` colidindo — daí o `role_`. A hidratação é a
    /// mesma de propósito: um papel é um papel, e duas leituras separadas
    /// divergiriam na primeira coluna nova.
    pub(crate) fn item_prefixed(row: &MySqlRow, prefix: &str) -> anyhow::Result<RoleViewItem> {
        Ok(RoleViewItem {
            id: Row::id(row, &format!("{prefix}id"))?,
            name: Row::text(row, &format!("{prefix}name"))?,
            user_count: Row::number(row, &format!("{prefix}user_count"))?,
            permissions: Row::permissions(row, &format!("{prefix}permissions"))?,
        })
    }
}
