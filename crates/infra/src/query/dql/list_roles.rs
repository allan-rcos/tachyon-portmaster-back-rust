//! A listagem paginada de papéis.

use anyhow::Context as _;
use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{RoleListView, RoleViewItem};
use crate::query::{Dql, SqlDql};

/// A listagem paginada de papéis.
pub fn list_roles(
    cursor: Option<String>,
    limit: Option<u32>,
    search: Option<&str>,
) -> impl SqlDql<View = RoleListView> {
    ListRoles {
        limit: Paging::effective_limit(limit),
        search: Paging::normalized_search(search),
        cursor,
    }
}

/// Uma linha de `roles` como a View a quer, com ou sem prefixo de `JOIN`.
///
/// Nas consultas de conta o papel chega aninhado no usuário, e as duas metades
/// da linha teriam `id` e `name` colidindo — daí o `role_`. A hidratação é a
/// mesma de propósito: um papel é um papel, e duas leituras separadas
/// divergiriam na primeira coluna nova.
pub(super) fn read_item(row: &Row, prefix: &str) -> anyhow::Result<RoleViewItem> {
    let id: i64 = Column::of(row, &format!("{prefix}id"))?;

    Ok(RoleViewItem {
        id: Codec::encode_id(id),
        name: Column::of(row, &format!("{prefix}name"))?,
        user_count: Column::of(row, &format!("{prefix}user_count"))?,
        permissions: read_permissions(row, &format!("{prefix}permissions"))?,
    })
}

/// A coluna `permissions`, que é um array JSON de slugs.
///
/// Um elemento que não seja texto é descartado em vez de derrubar a consulta:
/// permissão é lista de autorização, e uma entrada corrompida no meio não deve
/// impedir de ler as outras. Descartar aqui **restringe** o que o papel concede,
/// que é o lado seguro de errar.
pub(super) fn read_permissions(row: &Row, column: &str) -> anyhow::Result<Vec<String>> {
    let raw: Option<String> = Column::of(row, column)?;

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

/// A listagem de papéis.
struct ListRoles {
    /// O tamanho da página, já resolvido.
    limit: u32,
    /// O termo já reduzido à chave que as colunas `search_*` guardam.
    search: Option<String>,
    /// O cursor como o cliente o mandou.
    cursor: Option<String>,
}

impl ListRoles {
    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit.to_string()),
            ("search", self.search.clone().unwrap_or_default()),
        ])
    }

    /// A condição de busca, aplicada à página e à contagem.
    ///
    /// As duas precisam descrever o mesmo conjunto, e o nome do parâmetro é o
    /// mesmo nos dois lugares — ligado uma vez só.
    fn search_condition(&self, alias: &str) -> String {
        self.search.as_ref().map_or_else(String::new, |_| {
            format!(" AND {alias}search_name LIKE :search")
        })
    }
}

impl Dql for ListRoles {
    type View = RoleListView;

    fn cache_key(&self) -> String {
        format!(
            "list_roles:{}:{}:{}",
            self.limit,
            self.search.as_deref().unwrap_or_default(),
            self.cursor.as_deref().unwrap_or_default()
        )
    }
}

impl SqlDql for ListRoles {
    /// A contagem de usuários é sub-consulta correlacionada, e não `LEFT JOIN`
    /// com `GROUP BY`.
    ///
    /// Com o join, um papel sem usuário nenhum sumiria da contagem, e a
    /// agregação teria que abraçar todas as outras colunas só para sobreviver ao
    /// `GROUP BY`.
    fn build(&self) -> (String, Params) {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let sql = format!(
            "SELECT r.id, r.name, r.permissions, \
             (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count, \
             (SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL{total_search}) AS _total \
             FROM roles r \
             WHERE r.id > :last_id AND r.deleted_at IS NULL{page_search} \
             ORDER BY r.id ASC LIMIT :limit",
            total_search = self.search_condition(""),
            page_search = self.search_condition("r."),
        );

        let mut values = vec![
            ("last_id".to_owned(), Value::Int(last_id)),
            ("limit".to_owned(), Value::Int(i64::from(self.limit))),
        ];

        if let Some(term) = &self.search {
            values.push(("search".to_owned(), Value::from(Paging::like(term))));
        }

        (sql, values.into())
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row, "")?);
            last_id = Column::of(row, "id")?;
            total = Column::of(row, "_total")?;
        }

        Ok(RoleListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}
