//! A consulta de um papel pelo id.

use sqlx::mysql::{MySql, MySqlRow};
use sqlx::QueryBuilder;

use crate::entity::codec::Codec;
use crate::query::dql::list_roles::read_item;
use crate::query::views::RoleViewItem;
use crate::query::{Dql, SqlDql};

/// As colunas próprias do papel.
const COLUMNS: &str = "r.id, r.name, r.permissions";

/// Quantos usuários têm o papel.
const USER_COUNT: &str =
    "(SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count";

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
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(", ");
        builder.push(USER_COUNT);
        builder.push(" FROM roles r WHERE r.id = ");
        builder.push_bind(self.id);
        builder.push(" AND r.deleted_at IS NULL LIMIT 1");

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(|row| read_item(row, "")).transpose()
    }
}
