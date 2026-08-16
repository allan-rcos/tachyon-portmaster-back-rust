//! A consulta de um papel pelo id.

use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::dql::list_roles::read_item;
use crate::query::views::RoleViewItem;
use crate::query::{Dql, SqlDql};

/// Um papel pelo id.
pub fn get_role(id: &str) -> anyhow::Result<impl SqlDql<View = Option<RoleViewItem>>> {
    Ok(GetRole {
        id: Codec::decode_id(id)?,
    })
}

/// Um papel pelo id.
struct GetRole {
    /// O alvo, como `BIGINT`.
    id: i64,
}

impl Dql for GetRole {
    type View = Option<RoleViewItem>;

    fn cache_key(&self) -> String {
        format!("get_role:{}", self.id)
    }
}

impl SqlDql for GetRole {
    /// A contagem de usuários é sub-consulta correlacionada, e não `LEFT JOIN`
    /// com `GROUP BY`.
    ///
    /// Com o join, um papel sem usuário nenhum sumiria da contagem, e a
    /// agregação teria que abraçar todas as outras colunas só para sobreviver ao
    /// `GROUP BY`.
    fn build(&self) -> (String, Params) {
        let sql = "SELECT r.id, r.name, r.permissions, \
                   (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count \
                   FROM roles r WHERE r.id = :id AND r.deleted_at IS NULL LIMIT 1";

        (
            sql.to_owned(),
            vec![("id".to_owned(), Value::Int(self.id))].into(),
        )
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        rows.first().map(|row| read_item(row, "")).transpose()
    }
}
