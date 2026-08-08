//! A listagem de contêineres com carga e telemetria recente.

use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::container_reader::ContainerReader;
use crate::query::dql::paging::Paging;
use crate::query::params::SummaryListParams;
use crate::query::row::Row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{ContainerSummaryListView, ContainerSummaryViewItem};
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas que a View de contêiner precisa.
const COLUMNS: &str = "c.id, c.code, c.current_weight, c.max_capacity, c.status";

/// Quantos registros de telemetria a janela recente traz.
const RECENT_LOGS: u32 = 10;

/// A listagem de contêineres com carga e telemetria recente.
pub struct ListContainerSummariesDql {
    params: SummaryListParams,
    id: Option<i64>,
}

impl ListContainerSummariesDql {
    /// Monta a consulta.
    pub(crate) const fn new(params: SummaryListParams, id: Option<i64>) -> Self {
        Self { params, id }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit().to_string()),
            ("id", self.params.id.clone().unwrap_or_default()),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        Paging::effective_limit(self.params.limit)
    }
}

impl Dql for ListContainerSummariesDql {
    type View = ContainerSummaryListView;
}

impl SqlDql for ListContainerSummariesDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        // As duas coleções aninhadas viram JSON no próprio banco. A alternativa
        // — uma consulta por contêiner para a carga e outra para os logs —
        // custaria 2n idas ao banco por página.
        let manifest = "(SELECT JSON_ARRAYAGG(JSON_OBJECT( \
                        'product_id', ci.product_id, 'product_name', p.name, \
                        'quantity', ci.quantity, 'weight', ci.weight)) \
                        FROM container_items ci \
                        INNER JOIN products p ON p.id = ci.product_id \
                        WHERE ci.container_id = c.id) AS manifest_json";

        // O timestamp já sai como epoch em ms, na forma que a View guarda. Sair
        // como texto de data obrigaria a hidratação a interpretar formato de
        // data vindo de dentro de um JSON — duas conversões para chegar no mesmo
        // número.
        //
        // A janela dos recentes é uma sub-consulta escalar correlacionada e não
        // uma tabela derivada: o MariaDB não tem LATERAL, então uma derivada não
        // pode enxergar o `c.id` de fora. Limitar pelo id do n-ésimo mais novo
        // traz as mesmas linhas com uma correlação que o motor aceita.
        let logs = format!(
            "(SELECT JSON_ARRAYAGG(JSON_OBJECT( \
             'id', t.id, 'event', t.event, 'description', t.description, \
             'timestamp', CAST(UNIX_TIMESTAMP(t.timestamp) * 1000 AS SIGNED))) \
             FROM telemetry_logs t \
             WHERE t.container_id = c.id \
             AND t.id >= COALESCE((SELECT t2.id FROM telemetry_logs t2 \
             WHERE t2.container_id = c.id ORDER BY t2.id DESC LIMIT 1 OFFSET {}), 0)) AS logs_json",
            RECENT_LOGS - 1
        );

        let mut total = Select::from("containers")
            .column("COUNT(*)")
            .filter("deleted_at IS NULL", []);
        if let Some(id) = self.id {
            total = total.filter("id = ?", [Bind::Int(id)]);
        }

        let mut select = Select::from("containers c")
            .column(COLUMNS)
            .column(manifest)
            .column(logs)
            .column_bound(format!("({}) AS _total", total.to_sql()), total.binds())
            .filter("c.id > ?", [Bind::Int(last_id)])
            .filter("c.deleted_at IS NULL", [])
            .order_by("c.id ASC")
            .limit(limit);

        if let Some(id) = self.id {
            select = select.filter("c.id = ?", [Bind::Int(id)]);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(ContainerSummaryViewItem {
                container: ContainerReader::item(row)?,
                manifest: ContainerReader::manifest_of(
                    Row::opt_text(row, "manifest_json")?.as_deref(),
                )?,
                recent_logs: ContainerReader::logs_of(Row::opt_text(row, "logs_json")?.as_deref())?,
            });
            last_id = Row::number(row, "id")?;
            total = Row::number(row, "_total")?;
        }

        Ok(ContainerSummaryListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::codec::Codec;
    use crate::query::views::CargoItemView;
    use portmaster_domain::enums::TelemetryEvent;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_janela_de_telemetria_e_correlacionada_por_id() {
        // O MariaDB não tem LATERAL: uma tabela derivada não enxergaria `c.id`.
        let query = ListContainerSummariesDql::new(SummaryListParams::default(), None).build();

        assert!(
            query.sql.contains("LIMIT 1 OFFSET 9"),
            "a janela deveria limitar pelo décimo mais novo: {}",
            query.sql
        );
        assert!(
            query.sql.contains("COALESCE("),
            "faltou o caso de menos logs que o teto"
        );
    }

    #[test]
    fn manifesto_vazio_nao_e_erro() {
        // JSON_ARRAYAGG devolve NULL num contêiner sem carga, que é estado
        // normal.
        assert_eq!(ContainerReader::manifest_of(None).unwrap(), Vec::new());
    }

    #[test]
    fn o_manifesto_sai_do_json_agregado() {
        let json = r#"[{"product_id":1,"product_name":"Cimento","quantity":2.5,"weight":50.0}]"#;

        assert_eq!(
            ContainerReader::manifest_of(Some(json)).unwrap(),
            vec![CargoItemView {
                product_id: Codec::encode_id(1),
                product_name: "Cimento".into(),
                quantity: 2.5,
                weight: 50.0,
            }]
        );
    }

    #[test]
    fn evento_desconhecido_e_descartado_e_nao_aproximado() {
        // O campo do fio é um enum: não há valor que signifique "aconteceu algo,
        // mas nenhum destes".
        let json = r#"[{"id":1,"event":0,"description":null,"timestamp":1000},
                       {"id":2,"event":98,"description":null,"timestamp":2000}]"#;

        let logs = ContainerReader::logs_of(Some(json)).unwrap();

        assert_eq!(logs.len(), 1, "o evento fora da faixa deveria ter saído");
        assert_eq!(logs[0].event, TelemetryEvent::Load.as_i32());
        assert_eq!(logs[0].timestamp, 1000);
    }
}
