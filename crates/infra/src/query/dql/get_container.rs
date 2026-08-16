//! A consulta de um contêiner pelo id.

use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::dql::list_containers::read_item;
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
    /// As colunas são nomeadas em vez de `*`: a projeção é o contrato da
    /// hidratação, e um `SELECT *` faria uma coluna nova entrar na consulta sem
    /// que ninguém a pedisse.
    fn build(&self) -> (String, Params) {
        let sql = "SELECT c.id, c.code, c.current_weight, c.max_capacity, c.status \
                   FROM containers c WHERE c.id = :id AND c.deleted_at IS NULL LIMIT 1";

        (
            sql.to_owned(),
            vec![("id".to_owned(), Value::Int(self.id))].into(),
        )
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        rows.first().map(read_item).transpose()
    }
}
