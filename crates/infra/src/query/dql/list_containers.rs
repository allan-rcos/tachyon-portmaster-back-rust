//! A listagem paginada de contêineres.

use anyhow::{anyhow, Context as _};
use portmaster_domain::enums::ContainerStatus;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::entity::codec::Codec;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{ContainerListView, ContainerViewItem};
use crate::query::{Dql, SqlDql};

/// As colunas que a View de contêiner precisa.
pub(super) const COLUMNS: &str = "c.id, c.code, c.current_weight, c.max_capacity, c.status";

/// A listagem paginada de contêineres.
pub fn list_containers(
    cursor: Option<String>,
    limit: Option<u32>,
    search: Option<&str>,
    status: Option<ContainerStatus>,
    status_in: Vec<ContainerStatus>,
) -> impl SqlDql<View = ContainerListView> {
    ListContainers {
        limit: Paging::effective_limit(limit),
        search: Paging::normalized_search(search),
        status,
        status_in,
        cursor,
    }
}

/// Uma linha de `containers` como a View a quer.
pub(super) fn read_item(row: &MySqlRow) -> anyhow::Result<ContainerViewItem> {
    let status: i64 = row
        .try_get("status")
        .context("coluna `status` não veio como inteiro")?;
    let status = i32::try_from(status)
        .with_context(|| format!("coluna `status` guarda {status}, fora da faixa"))?;

    ContainerStatus::from_i32(status)
        .ok_or_else(|| anyhow!("{status} não corresponde a variante nenhuma de ContainerStatus"))?;

    Ok(ContainerViewItem {
        id: Codec::encode_id(
            row.try_get("id")
                .context("coluna `id` não veio como inteiro")?,
        ),
        code: row
            .try_get("code")
            .context("coluna `code` não veio como texto")?,
        current_weight: row
            .try_get("current_weight")
            .context("coluna `current_weight` não veio como real")?,
        max_capacity: row
            .try_get("max_capacity")
            .context("coluna `max_capacity` não veio como real")?,
        status,
    })
}

/// A listagem de contêineres.
struct ListContainers {
    /// O tamanho da página, já resolvido.
    limit: u32,
    /// O termo já reduzido à chave que as colunas `search_*` guardam.
    search: Option<String>,
    /// O status exato pedido, se houver.
    status: Option<ContainerStatus>,
    /// O conjunto de status aceitos, se houver.
    status_in: Vec<ContainerStatus>,
    /// O cursor como o cliente o mandou.
    cursor: Option<String>,
}

impl ListContainers {
    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit.to_string()),
            ("search", self.search.clone().unwrap_or_default()),
            (
                "status",
                self.status
                    .map(|status| status.as_i32().to_string())
                    .unwrap_or_default(),
            ),
            (
                "status_in",
                self.status_in
                    .iter()
                    .map(|status| status.as_i32().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ])
    }

    /// Escreve as condições que a página e a contagem aplicam igualmente.
    ///
    /// Uma função só, chamada duas vezes com prefixos diferentes: é o que
    /// garante que o total descreva o mesmo conjunto que a página percorre.
    /// Duas listas escritas em paralelo divergiriam no primeiro filtro novo.
    fn push_conditions(&self, builder: &mut QueryBuilder<MySql>, alias: &str) {
        if let Some(term) = &self.search {
            builder.push(format!(" AND {alias}search_code LIKE "));
            builder.push_bind(Paging::like(term));
        }

        if let Some(status) = self.status {
            builder.push(format!(" AND {alias}status = "));
            builder.push_bind(i64::from(status.as_i32()));
        }

        if !self.status_in.is_empty() {
            builder.push(format!(" AND {alias}status IN ("));

            let mut separated = builder.separated(", ");
            for status in &self.status_in {
                separated.push_bind(i64::from(status.as_i32()));
            }

            builder.push(")");
        }
    }
}

impl Dql for ListContainers {
    type View = ContainerListView;

    fn cache_key(&self) -> String {
        format!(
            "list_containers:{}:{}:{}:{}:{}",
            self.limit,
            self.search.as_deref().unwrap_or_default(),
            self.status.map_or(-1, ContainerStatus::as_i32),
            self.status_in
                .iter()
                .map(|status| status.as_i32().to_string())
                .collect::<Vec<_>>()
                .join(","),
            self.cursor.as_deref().unwrap_or_default()
        )
    }
}

impl SqlDql for ListContainers {
    fn build(&self) -> QueryBuilder<MySql> {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);

        builder.push(", (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL");
        self.push_conditions(&mut builder, "");
        builder.push(") AS _total");

        builder.push(" FROM containers c WHERE c.id > ");
        builder.push_bind(last_id);
        builder.push(" AND c.deleted_at IS NULL");
        self.push_conditions(&mut builder, "c.");

        builder.push(" ORDER BY c.id ASC LIMIT ");
        builder.push_bind(i64::from(self.limit));

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row)?);
            last_id = row
                .try_get("id")
                .context("coluna `id` não veio como inteiro")?;
            total = row
                .try_get("_total")
                .context("coluna `_total` não veio como inteiro")?;
        }

        Ok(ContainerListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
#[path = "tests/list_containers_test.rs"]
mod tests;
