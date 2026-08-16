//! A listagem paginada de contêineres.

use anyhow::{anyhow, Context as _};
use mysql_async::{Params, Row, Value};
use portmaster_domain::enums::ContainerStatus;

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{ContainerListView, ContainerViewItem};
use crate::query::{Dql, SqlDql};

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
pub(super) fn read_item(row: &Row) -> anyhow::Result<ContainerViewItem> {
    let status: i64 = Column::of(row, "status")?;
    let status = i32::try_from(status)
        .with_context(|| format!("coluna `status` guarda {status}, fora da faixa"))?;

    ContainerStatus::from_i32(status)
        .ok_or_else(|| anyhow!("{status} não corresponde a variante nenhuma de ContainerStatus"))?;

    Ok(ContainerViewItem {
        id: Codec::encode_id(Column::of(row, "id")?),
        code: Column::of(row, "code")?,
        current_weight: Column::of(row, "current_weight")?,
        max_capacity: Column::of(row, "max_capacity")?,
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

    /// As condições que a página e a contagem aplicam igualmente.
    ///
    /// Uma função só, chamada duas vezes com prefixos diferentes: é o que
    /// garante que o total descreva o mesmo conjunto que a página percorre.
    /// Duas listas escritas em paralelo divergiriam no primeiro filtro novo.
    ///
    /// O mesmo nome de parâmetro aparece nas duas, e é ligado uma vez só — o
    /// driver o repete em cada ocorrência do texto.
    fn conditions(&self, alias: &str) -> String {
        let mut parts = Vec::new();

        if self.search.is_some() {
            parts.push(format!(" AND {alias}search_code LIKE :search"));
        }

        if self.status.is_some() {
            parts.push(format!(" AND {alias}status = :status"));
        }

        if !self.status_in.is_empty() {
            let names = (0..self.status_in.len())
                .map(|position| format!(":status_{position}"))
                .collect::<Vec<_>>()
                .join(", ");

            parts.push(format!(" AND {alias}status IN ({names})"));
        }

        parts.concat()
    }

    /// Os valores que o texto da consulta nomeia.
    ///
    /// Um conjunto de status de tamanho variável não tem como ser um nome só:
    /// os nomes saem numerados, no mesmo laço que os liga, de modo que texto e
    /// valores não têm como divergir.
    fn params(&self, last_id: i64) -> Params {
        let mut values = vec![
            ("last_id".to_owned(), Value::Int(last_id)),
            ("limit".to_owned(), Value::Int(i64::from(self.limit))),
        ];

        if let Some(term) = &self.search {
            values.push(("search".to_owned(), Value::from(Paging::like(term))));
        }

        if let Some(status) = self.status {
            values.push(("status".to_owned(), Value::Int(i64::from(status.as_i32()))));
        }

        for (position, status) in self.status_in.iter().enumerate() {
            values.push((
                format!("status_{position}"),
                Value::Int(i64::from(status.as_i32())),
            ));
        }

        values.into()
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
    /// As colunas são nomeadas em vez de `*`: a projeção é o contrato da
    /// hidratação, e um `SELECT *` faria uma coluna nova entrar na consulta sem
    /// que ninguém a pedisse.
    fn build(&self) -> (String, Params) {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let sql = format!(
            "SELECT c.id, c.code, c.current_weight, c.max_capacity, c.status, \
             (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL{total_conditions}) AS _total \
             FROM containers c \
             WHERE c.id > :last_id AND c.deleted_at IS NULL{page_conditions} \
             ORDER BY c.id ASC LIMIT :limit",
            total_conditions = self.conditions(""),
            page_conditions = self.conditions("c."),
        );

        (sql, self.params(last_id))
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row)?);
            last_id = Column::of(row, "id")?;
            total = Column::of(row, "_total")?;
        }

        Ok(ContainerListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}
