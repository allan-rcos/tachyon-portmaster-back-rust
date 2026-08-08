//! A consulta de um contêiner pelo id.

use crate::query::dql::container_reader::ContainerReader;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::ContainerViewItem;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas que a View de contêiner precisa.
const COLUMNS: &str = "c.id, c.code, c.current_weight, c.max_capacity, c.status";

/// Um contêiner pelo id.
pub struct GetContainerDql {
    /// O alvo, como `BIGINT` — o base62 já foi decodificado pelo caso de uso.
    id: i64,
}

impl GetContainerDql {
    /// Monta a consulta.
    pub(crate) const fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Dql for GetContainerDql {
    type View = Option<ContainerViewItem>;
}

impl SqlDql for GetContainerDql {
    fn build(&self) -> SqlQuery {
        Select::from("containers c")
            .column(COLUMNS)
            .filter("c.id = ?", [Bind::Int(self.id)])
            .filter("c.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(ContainerReader::item).transpose()
    }
}
