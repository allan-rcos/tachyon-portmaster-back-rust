//! As consultas de papel.

use sqlx::mysql::MySqlRow;

use super::{effective_limit, like, normalized_search};
use crate::query::cursor::{filters, Cursor, CursorFilters};
use crate::query::row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{RoleListView, RoleViewItem};
use crate::query::{Dql, ListParams, SqlDql};

/// As colunas próprias do papel.
const COLUMNS: &str = "r.id, r.name, r.permissions";

/// Quantos usuários têm o papel.
///
/// Sub-consulta correlacionada e não `LEFT JOIN` + `GROUP BY`: com o join, um
/// papel sem usuário sumiria da contagem, e a agregação teria que abraçar todas
/// as outras colunas só para sobreviver ao `GROUP BY`.
const USER_COUNT: &str =
    "(SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count";

/// Um papel pelo id.
pub(crate) struct GetRoleDql {
    id: i64,
}

impl GetRoleDql {
    /// Monta a consulta.
    pub(crate) fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Dql for GetRoleDql {
    type View = Option<RoleViewItem>;
}

impl SqlDql for GetRoleDql {
    fn build(&self) -> SqlQuery {
        Select::from("roles r")
            .column(COLUMNS)
            .column(USER_COUNT)
            .filter("r.id = ?", [Bind::Int(self.id)])
            .filter("r.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(item).transpose()
    }
}

/// A listagem de papéis.
pub(crate) struct ListRolesDql {
    params: ListParams,
}

impl ListRolesDql {
    /// Monta a consulta.
    pub(crate) fn new(params: ListParams) -> Self {
        Self { params }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        filters([
            ("limit", self.limit().to_string()),
            (
                "search",
                normalized_search(self.params.search.as_deref()).unwrap_or_default(),
            ),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        effective_limit(self.params.limit)
    }
}

impl Dql for ListRolesDql {
    type View = RoleListView;
}

impl SqlDql for ListRolesDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let search = normalized_search(self.params.search.as_deref());
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        let (total_sql, total_binds) = match &search {
            Some(term) => (
                "(SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total",
                vec![Bind::Text(like(term))],
            ),
            None => (
                "(SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL) AS _total",
                Vec::new(),
            ),
        };

        let mut select = Select::from("roles r")
            .column(COLUMNS)
            .column(USER_COUNT)
            .column_bound(total_sql, total_binds)
            .filter("r.id > ?", [Bind::Int(last_id)])
            .filter("r.deleted_at IS NULL", [])
            .order_by("r.id ASC")
            .limit(limit);

        if let Some(term) = &search {
            select = select.filter("r.search_name LIKE ?", [Bind::Text(like(term))]);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(item(row)?);
            last_id = row::number(row, "id")?;
            total = row::number(row, "_total")?;
        }

        Ok(RoleListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

/// Uma linha de `roles` como a View a quer.
pub(crate) fn item(row: &MySqlRow) -> anyhow::Result<RoleViewItem> {
    item_prefixed(row, "")
}

/// A mesma leitura, quando as colunas do papel vêm prefixadas.
///
/// Nas consultas de conta o papel chega aninhado no usuário, e as duas metades
/// da linha teriam `id` e `name` colidindo — daí o `role_`. A hidratação é a
/// mesma de propósito: um papel é um papel, e duas leituras separadas
/// divergiriam na primeira coluna nova.
pub(crate) fn item_prefixed(row: &MySqlRow, prefix: &str) -> anyhow::Result<RoleViewItem> {
    Ok(RoleViewItem {
        id: row::id(row, &format!("{prefix}id"))?,
        name: row::text(row, &format!("{prefix}name"))?,
        user_count: row::number(row, &format!("{prefix}user_count"))?,
        permissions: row::permissions(row, &format!("{prefix}permissions"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_contagem_de_usuarios_e_correlacionada() {
        // Com LEFT JOIN + GROUP BY, um papel sem nenhum usuário sairia da
        // listagem em vez de sair com zero.
        let query = GetRoleDql::new(7).build();

        assert_eq!(
            query.sql,
            "SELECT r.id, r.name, r.permissions, \
             (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count \
             FROM roles r WHERE r.id = ? AND r.deleted_at IS NULL LIMIT 1"
        );
        assert_eq!(query.binds, vec![Bind::Int(7)]);
    }

    #[test]
    fn a_busca_entra_na_pagina_e_na_contagem() {
        let query = ListRolesDql::new(ListParams {
            search: Some("Operador".into()),
            ..ListParams::default()
        })
        .build();

        assert_eq!(
            query.binds,
            vec![
                Bind::Text("%operador%".into()),
                Bind::Int(0),
                Bind::Text("%operador%".into()),
            ]
        );
    }
}
