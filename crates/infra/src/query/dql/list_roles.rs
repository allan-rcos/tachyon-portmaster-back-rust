//! A listagem paginada de papéis.

use anyhow::Context as _;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::entity::codec::Codec;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{RoleListView, RoleViewItem};
use crate::query::{Dql, SqlDql};

/// As colunas próprias do papel.
const COLUMNS: &str = "r.id, r.name, r.permissions";

/// Quantos usuários têm o papel.
///
/// Sub-consulta correlacionada e não `LEFT JOIN` + `GROUP BY`: com o join, um
/// papel sem usuário sumiria da contagem, e a agregação teria que abraçar todas
/// as outras colunas só para sobreviver ao `GROUP BY`.
const USER_COUNT: &str =
    "(SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count";

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
pub(super) fn read_item(row: &MySqlRow, prefix: &str) -> anyhow::Result<RoleViewItem> {
    let id: i64 = row
        .try_get(&*format!("{prefix}id"))
        .context("coluna de id do papel não veio como inteiro")?;

    Ok(RoleViewItem {
        id: Codec::encode_id(id),
        name: row
            .try_get(&*format!("{prefix}name"))
            .context("coluna de nome do papel não veio como texto")?,
        user_count: row
            .try_get(&*format!("{prefix}user_count"))
            .context("coluna de contagem do papel não veio como inteiro")?,
        permissions: read_permissions(row, &format!("{prefix}permissions"))?,
    })
}

/// A coluna `permissions`, que é um array JSON de slugs.
///
/// Um elemento que não seja texto é descartado em vez de derrubar a consulta:
/// permissão é lista de autorização, e uma entrada corrompida no meio não deve
/// impedir de ler as outras. Descartar aqui **restringe** o que o papel concede,
/// que é o lado seguro de errar.
pub(super) fn read_permissions(row: &MySqlRow, column: &str) -> anyhow::Result<Vec<String>> {
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
    fn build(&self) -> QueryBuilder<MySql> {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(", ");
        builder.push(USER_COUNT);

        builder.push(", (SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL");
        if let Some(term) = &self.search {
            builder.push(" AND search_name LIKE ");
            builder.push_bind(Paging::like(term));
        }
        builder.push(") AS _total");

        builder.push(" FROM roles r WHERE r.id > ");
        builder.push_bind(last_id);
        builder.push(" AND r.deleted_at IS NULL");

        if let Some(term) = &self.search {
            builder.push(" AND r.search_name LIKE ");
            builder.push_bind(Paging::like(term));
        }

        builder.push(" ORDER BY r.id ASC LIMIT ");
        builder.push_bind(i64::from(self.limit));

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row, "")?);
            last_id = row
                .try_get("id")
                .context("coluna `id` não veio como inteiro")?;
            total = row
                .try_get("_total")
                .context("coluna `_total` não veio como inteiro")?;
        }

        Ok(RoleListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
#[path = "tests/list_roles_test.rs"]
mod tests;
