//! A consulta de um contêiner pelo id.

use sqlx::mysql::{MySql, MySqlRow};
use sqlx::QueryBuilder;

use crate::entity::codec::Codec;
use crate::query::dql::list_containers::{read_item, COLUMNS};
use crate::query::views::ContainerViewItem;
use crate::query::{Dql, SqlDql};

/// Um contêiner pelo id.
pub fn get_container(id: &str) -> anyhow::Result<impl SqlDql<View = Option<ContainerViewItem>>> {
    Ok(GetContainer {
        id: Codec::decode_id(id)?,
    })
}

/// Um contêiner pelo id.
struct GetContainer {
    /// O alvo, como `BIGINT`.
    id: i64,
}

impl Dql for GetContainer {
    type View = Option<ContainerViewItem>;

    fn cache_key(&self) -> String {
        format!("get_container:{}", self.id)
    }
}

impl SqlDql for GetContainer {
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(" FROM containers c WHERE c.id = ");
        builder.push_bind(self.id);
        builder.push(" AND c.deleted_at IS NULL LIMIT 1");

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(read_item).transpose()
    }
}

#[cfg(test)]
#[path = "tests/get_container_test.rs"]
mod tests;
