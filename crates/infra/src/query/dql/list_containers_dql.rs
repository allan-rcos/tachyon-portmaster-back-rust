//! A listagem paginada de contêineres.

use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::container_reader::ContainerReader;
use crate::query::dql::paging::Paging;
use crate::query::params::ContainerListParams;
use crate::query::row::Row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::ContainerListView;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas que a View de contêiner precisa.
const COLUMNS: &str = "c.id, c.code, c.current_weight, c.max_capacity, c.status";

/// A listagem de contêineres.
pub struct ListContainersDql {
    /// Cursor, limite, busca e os filtros de status.
    params: ContainerListParams,
}

impl ListContainersDql {
    /// Monta a consulta.
    pub(crate) const fn new(params: ContainerListParams) -> Self {
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
            (
                "status",
                self.params
                    .status
                    .map(|s| s.as_i32().to_string())
                    .unwrap_or_default(),
            ),
            (
                "status_in",
                self.params
                    .status_in
                    .iter()
                    .map(|s| s.as_i32().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        Paging::effective_limit(self.params.limit)
    }

    /// As condições que a página e a contagem têm que aplicar igualmente.
    ///
    /// Uma lista só, consumida duas vezes com prefixos diferentes: é o que
    /// garante que o total descreva o mesmo conjunto que a página percorre. Duas
    /// listas escritas em paralelo divergiriam no primeiro filtro novo.
    fn conditions(&self, alias: &str) -> Vec<(String, Vec<Bind>)> {
        let mut conditions = Vec::new();

        if let Some(term) = Paging::normalized_search(self.params.search.as_deref()) {
            conditions.push((
                format!("{alias}search_code LIKE ?"),
                vec![Bind::Text(Paging::like(&term))],
            ));
        }

        if let Some(status) = self.params.status {
            conditions.push((
                format!("{alias}status = ?"),
                vec![Bind::Int(status.as_i32().into())],
            ));
        }

        if !self.params.status_in.is_empty() {
            let placeholders = vec!["?"; self.params.status_in.len()].join(", ");
            conditions.push((
                format!("{alias}status IN ({placeholders})"),
                self.params
                    .status_in
                    .iter()
                    .map(|status| Bind::Int(status.as_i32().into()))
                    .collect(),
            ));
        }

        conditions
    }
}

impl Dql for ListContainersDql {
    type View = ContainerListView;
}

impl SqlDql for ListContainersDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        let mut total = Select::from("containers")
            .column("COUNT(*)")
            .filter("deleted_at IS NULL", []);
        for (condition, binds) in self.conditions("") {
            total = total.filter(condition, binds);
        }

        let mut select = Select::from("containers c")
            .column(COLUMNS)
            .column_bound(format!("({}) AS _total", total.to_sql()), total.binds())
            .filter("c.id > ?", [Bind::Int(last_id)])
            .filter("c.deleted_at IS NULL", [])
            .order_by("c.id ASC")
            .limit(limit);

        for (condition, binds) in self.conditions("c.") {
            select = select.filter(condition, binds);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(ContainerReader::item(row)?);
            last_id = Row::number(row, "id")?;
            total = Row::number(row, "_total")?;
        }

        Ok(ContainerListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::params::ContainerListParams;
    use portmaster_domain::enums::ContainerStatus;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_conjunto_de_status_vira_um_in_com_um_placeholder_por_valor() {
        let query = ListContainersDql::new(ContainerListParams {
            status_in: vec![ContainerStatus::Loading, ContainerStatus::Sealed],
            ..ContainerListParams::default()
        })
        .build();

        assert!(
            query.sql.contains("c.status IN (?, ?)"),
            "esperava um placeholder por status: {}",
            query.sql
        );
        assert_eq!(
            query.binds,
            vec![
                // A contagem primeiro — os `?` dela saem antes no texto.
                Bind::Int(1),
                Bind::Int(2),
                // Depois o cursor e os mesmos status, agora na página.
                Bind::Int(0),
                Bind::Int(1),
                Bind::Int(2),
            ]
        );
    }

    /// Se divergirem, o cliente recebe uma página de três itens dizendo que há
    /// quatrocentos.
    #[test]
    fn a_contagem_repete_exatamente_os_filtros_da_pagina() {
        let dql = ListContainersDql::new(ContainerListParams {
            search: Some("BR-99".into()),
            status: Some(ContainerStatus::InTransit),
            ..ContainerListParams::default()
        });

        let da_pagina: Vec<_> = dql.conditions("c.").into_iter().map(|(_, b)| b).collect();
        let da_contagem: Vec<_> = dql.conditions("").into_iter().map(|(_, b)| b).collect();

        assert_eq!(da_pagina, da_contagem);
    }
}
