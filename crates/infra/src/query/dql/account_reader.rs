//! A leitura dos papéis de um usuário a partir das linhas do `JOIN`.

use sqlx::mysql::MySqlRow;

use crate::query::dql::role_reader::RoleReader;
use crate::query::row::Row;
use crate::query::views::RoleViewItem;

/// Lê a lista de papéis que veio repetida nas linhas de um usuário.
pub(crate) struct AccountReader;

impl AccountReader {
    /// Os papéis presentes num conjunto de linhas do mesmo usuário.
    pub(crate) fn roles_of(rows: &[MySqlRow]) -> anyhow::Result<Vec<RoleViewItem>> {
        let mut roles = Vec::with_capacity(rows.len());

        for row in rows {
            if let Some(role) = Self::role_of(row)? {
                roles.push(role);
            }
        }

        Ok(roles)
    }

    ///
    /// `role_id` nulo é a marca de "este usuário não tem papel": o join não
    /// achou par, e as demais colunas de papel vêm nulas junto.
    /// O papel de uma linha, ou `None` no lado vazio do `LEFT JOIN`.
    pub(crate) fn role_of(row: &MySqlRow) -> anyhow::Result<Option<RoleViewItem>> {
        if Row::opt_id(row, "role_id")?.is_none() {
            return Ok(None);
        }

        RoleReader::item_prefixed(row, "role_").map(Some)
    }
}
