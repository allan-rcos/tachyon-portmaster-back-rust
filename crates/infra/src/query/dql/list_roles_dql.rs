//! A listagem paginada de papéis.

use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::dql::role_reader::RoleReader;
use crate::query::params::ListParams;
use crate::query::row::Row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::RoleListView;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas próprias do papel.
const COLUMNS: &str = "r.id, r.name, r.permissions";

/// Quantos usuários têm o papel.
///
/// Sub-consulta correlacionada e não `LEFT JOIN` + `GROUP BY`: com o join, um
/// papel sem usuário sumiria da contagem, e a agregação teria que abraçar todas
/// as outras colunas só para sobreviver ao `GROUP BY`.
const USER_COUNT: &str =
    "(SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count";

/// A listagem de papéis.
pub struct ListRolesDql {
    /// Cursor, limite e busca da página pedida.
    params: ListParams,
}

impl ListRolesDql {
    /// Monta a consulta.
    pub(crate) const fn new(params: ListParams) -> Self {
        Self { params }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit().to_string()),
            (
                "search",
                Paging::normalized_search(self.params.search.as_deref()).unwrap_or_default(),
            ),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        Paging::effective_limit(self.params.limit)
    }
}

impl Dql for ListRolesDql {
    type View = RoleListView;
}

impl SqlDql for ListRolesDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let search = Paging::normalized_search(self.params.search.as_deref());
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        let (total_sql, total_binds) = match &search {
            Some(term) => (
                "(SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total",
                vec![Bind::Text(Paging::like(term))],
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
            select = select.filter("r.search_name LIKE ?", [Bind::Text(Paging::like(term))]);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(RoleReader::item(row)?);
            last_id = Row::number(row, "id")?;
            total = Row::number(row, "_total")?;
        }

        Ok(RoleListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::dql::get_role_dql::GetRoleDql;
    use crate::query::sql::Bind;
    use pretty_assertions::assert_eq;

    /// Com LEFT JOIN + GROUP BY, um papel sem nenhum usuário sairia da
    /// listagem em vez de sair com zero.
    #[test]
    fn a_contagem_de_usuarios_e_correlacionada() {
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
