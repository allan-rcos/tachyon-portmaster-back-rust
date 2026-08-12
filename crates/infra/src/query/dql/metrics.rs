//! O painel do pátio.

use anyhow::Context as _;
use portmaster_domain::enums::ContainerStatus;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::query::views::{MetricsView, OccupancyView};
use crate::query::{Dql, SqlDql};

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
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new(
            "SELECT (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL) AS total_containers",
        );

        builder.push(", (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL AND status <> ");
        builder.push_bind(i64::from(ContainerStatus::Empty.as_i32()));
        builder.push(") AS active_containers");

        builder.push(
            ", (SELECT COALESCE(SUM(current_weight), 0) FROM containers WHERE deleted_at IS NULL) AS yard_load",
        );
        builder.push(
            ", (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL) AS registered_products",
        );

        for (status, alias) in [
            (ContainerStatus::Empty, "occ_empty"),
            (ContainerStatus::Loading, "occ_loading"),
            (ContainerStatus::Sealed, "occ_sealed"),
            (ContainerStatus::InTransit, "occ_in_transit"),
        ] {
            builder
                .push(", (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL AND status = ");
            builder.push_bind(i64::from(status.as_i32()));
            builder.push(format!(") AS {alias}"));
        }

        builder.push(" FROM DUAL");

        builder
    }

    /// Um `SELECT` de agregações sempre devolve exatamente uma linha.
    ///
    /// Sem nenhuma, o painel zerado é a leitura honesta — não há o que reportar.
    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let Some(row) = rows.first() else {
            return Ok(MetricsView::default());
        };

        let count = |column: &str| -> anyhow::Result<i64> {
            row.try_get(column)
                .with_context(|| format!("coluna `{column}` não veio como inteiro"))
        };

        Ok(MetricsView {
            active_containers: count("active_containers")?,
            total_containers: count("total_containers")?,
            yard_load: row
                .try_get("yard_load")
                .context("coluna `yard_load` não veio como real")?,
            registered_products: count("registered_products")?,
            occupancy: OccupancyView {
                empty: count("occ_empty")?,
                loading: count("occ_loading")?,
                sealed: count("occ_sealed")?,
                in_transit: count("occ_in_transit")?,
            },
        })
    }
}

#[cfg(test)]
#[path = "tests/metrics_test.rs"]
mod tests;
