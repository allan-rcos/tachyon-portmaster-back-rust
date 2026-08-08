//! A consulta de um papel pelo id.

use crate::query::dql::role_reader::RoleReader;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::RoleViewItem;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas próprias do papel.
const COLUMNS: &str = "r.id, r.name, r.permissions";

/// Quantos usuários têm o papel.
///
/// Sub-consulta correlacionada e não `LEFT JOIN` + `GROUP BY`: com o join, um
/// papel sem usuário sumiria da contagem, e a agregação teria que abraçar todas
/// as outras colunas só para sobreviver ao `GROUP BY`.
const USER_COUNT: &str =
    "(SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count";

/// Um papel pelo id.
pub struct GetRoleDql {
    /// O alvo, como `BIGINT` — o base62 já foi decodificado pelo caso de uso.
    id: i64,
}

impl GetRoleDql {
    /// Monta a consulta.
    pub(crate) const fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Dql for GetRoleDql {
    type View = Option<RoleViewItem>;
}

impl SqlDql for GetRoleDql {
    fn build(&self) -> SqlQuery {
        Select::from("roles r")
            .column(COLUMNS)
            .column(USER_COUNT)
            .filter("r.id = ?", [Bind::Int(self.id)])
            .filter("r.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(RoleReader::item).transpose()
    }
}
