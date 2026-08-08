//! O painel do pátio.

use portmaster_domain::enums::ContainerStatus;
use sqlx::mysql::MySqlRow;

use crate::query::row::Row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{MetricsView, OccupancyView};
use crate::query::{Dql, SqlDql};

/// As oito agregações do painel.
pub struct MetricsDql;

impl MetricsDql {
    /// Monta a consulta.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl Dql for MetricsDql {
    type View = MetricsView;
}

impl SqlDql for MetricsDql {
    fn build(&self) -> SqlQuery {
        // Oito sub-consultas escalares num `SELECT` sem `FROM`: uma ida ao banco
        // devolve a linha inteira do painel. Oito consultas separadas dariam o
        // mesmo número por oito vezes o custo de rede.
        //
        // Os índices de status são bindados, e não interpolados, mesmo sendo
        // constantes nossas. Interpolar número em SQL é o hábito que um dia
        // encontra um valor que não é constante.
        let occupancy = |status: ContainerStatus, alias: &str| {
            (
                format!("(SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL AND status = ?) AS {alias}"),
                vec![Bind::Int(status.as_i32().into())],
            )
        };

        let (empty, empty_bind) = occupancy(ContainerStatus::Empty, "occ_empty");
        let (loading, loading_bind) = occupancy(ContainerStatus::Loading, "occ_loading");
        let (sealed, sealed_bind) = occupancy(ContainerStatus::Sealed, "occ_sealed");
        let (in_transit, in_transit_bind) = occupancy(ContainerStatus::InTransit, "occ_in_transit");

        Select::from("DUAL")
            .column("(SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL) AS total_containers")
            .column_bound(
                "(SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL AND status <> ?) AS active_containers",
                [Bind::Int(ContainerStatus::Empty.as_i32().into())],
            )
            .column(
                "(SELECT COALESCE(SUM(current_weight), 0) FROM containers WHERE deleted_at IS NULL) AS yard_load",
            )
            .column("(SELECT COUNT(*) FROM products WHERE deleted_at IS NULL) AS registered_products")
            .column_bound(empty, empty_bind)
            .column_bound(loading, loading_bind)
            .column_bound(sealed, sealed_bind)
            .column_bound(in_transit, in_transit_bind)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        // Um `SELECT` de agregações sempre devolve exatamente uma linha. Sem
        // nenhuma, o painel zerado é a leitura honesta — não há o que reportar.
        let Some(row) = rows.first() else {
            return Ok(MetricsView::default());
        };

        Ok(MetricsView {
            active_containers: Row::number(row, "active_containers")?,
            total_containers: Row::number(row, "total_containers")?,
            yard_load: Row::real(row, "yard_load")?,
            registered_products: Row::number(row, "registered_products")?,
            occupancy: OccupancyView {
                empty: Row::number(row, "occ_empty")?,
                loading: Row::number(row, "occ_loading")?,
                sealed: Row::number(row, "occ_sealed")?,
                in_transit: Row::number(row, "occ_in_transit")?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_painel_sai_de_uma_ida_so_ao_banco() {
        let query = MetricsDql::new().build();

        assert_eq!(
            query.sql.matches("AS ").count(),
            8,
            "as oito agregações deveriam sair na mesma linha: {}",
            query.sql
        );
    }

    #[test]
    fn os_status_sao_bindados_e_na_ordem_das_colunas() {
        let query = MetricsDql::new().build();

        assert_eq!(
            query.binds,
            vec![
                // active_containers (<> Empty), depois a ocupação em ordem.
                Bind::Int(0),
                Bind::Int(0),
                Bind::Int(1),
                Bind::Int(2),
                Bind::Int(3),
            ]
        );
    }

    #[test]
    fn sem_linha_o_painel_sai_zerado() {
        assert_eq!(
            MetricsDql::new().read(Vec::new()).unwrap(),
            MetricsView::default()
        );
    }
}
