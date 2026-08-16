//! O painel do pátio.

use mysql_async::{Params, Row, Value};
use portmaster_domain::enums::ContainerStatus;

use crate::query::column::Column;
use crate::query::views::{MetricsView, OccupancyView};
use crate::query::{Dql, SqlDql};

/// Os quatro status contados, e o nome que cada contagem recebe na linha.
///
/// O nome da coluna é também o nome do parâmetro que a filtra: são a mesma
/// contagem vista dos dois lados, e um par a menos para manter em sincronia.
const OCCUPANCY: &[(ContainerStatus, &str)] = &[
    (ContainerStatus::Empty, "occ_empty"),
    (ContainerStatus::Loading, "occ_loading"),
    (ContainerStatus::Sealed, "occ_sealed"),
    (ContainerStatus::InTransit, "occ_in_transit"),
];

/// O painel do pátio, numa ida só ao banco.
pub fn metrics() -> impl SqlDql<View = MetricsView> {
    Metrics
}

/// As oito agregações do painel.
struct Metrics;

impl Dql for Metrics {
    type View = MetricsView;

    fn cache_key(&self) -> String {
        "metrics".to_owned()
    }
}

impl SqlDql for Metrics {
    /// Oito sub-consultas escalares num `SELECT` sem `FROM`: uma ida ao banco
    /// devolve a linha inteira do painel.
    ///
    /// Oito consultas separadas dariam o mesmo número por oito vezes o custo de
    /// rede. Os índices de status são bindados, e não interpolados, mesmo sendo
    /// constantes nossas. Interpolar número em SQL é o hábito que um dia
    /// encontra um valor que não é constante.
    fn build(&self) -> (String, Params) {
        let occupancy = OCCUPANCY
            .iter()
            .map(|(_, alias)| {
                format!(
                    ", (SELECT COUNT(*) FROM containers \
                     WHERE deleted_at IS NULL AND status = :{alias}) AS {alias}"
                )
            })
            .collect::<Vec<_>>()
            .concat();

        let sql = format!(
            "SELECT \
             (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL) AS total_containers, \
             (SELECT COUNT(*) FROM containers \
              WHERE deleted_at IS NULL AND status <> :empty) AS active_containers, \
             (SELECT COALESCE(SUM(current_weight), 0) \
              FROM containers WHERE deleted_at IS NULL) AS yard_load, \
             (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL) AS registered_products\
             {occupancy}"
        );

        let mut values = vec![(
            "empty".to_owned(),
            Value::Int(i64::from(ContainerStatus::Empty.as_i32())),
        )];

        for (status, alias) in OCCUPANCY {
            values.push(((*alias).to_owned(), Value::Int(i64::from(status.as_i32()))));
        }

        (sql, values.into())
    }

    /// Um `SELECT` de agregações sempre devolve exatamente uma linha.
    ///
    /// Sem nenhuma, o painel zerado é a leitura honesta — não há o que reportar.
    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let Some(row) = rows.first() else {
            return Ok(MetricsView::default());
        };

        Ok(MetricsView {
            active_containers: Column::of(row, "active_containers")?,
            total_containers: Column::of(row, "total_containers")?,
            yard_load: Column::of(row, "yard_load")?,
            registered_products: Column::of(row, "registered_products")?,
            occupancy: OccupancyView {
                empty: Column::of(row, "occ_empty")?,
                loading: Column::of(row, "occ_loading")?,
                sealed: Column::of(row, "occ_sealed")?,
                in_transit: Column::of(row, "occ_in_transit")?,
            },
        })
    }
}
