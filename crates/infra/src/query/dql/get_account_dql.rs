//! A consulta de um usuário com os papéis dele.

use crate::query::dql::account_reader::AccountReader;
use crate::query::row::Row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::AccountView;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// A projeção comum às duas consultas.
///
/// Usuário e papel na mesma linha, com o papel prefixado porque `id` e `name`
/// existem dos dois lados.
const COLUMNS: &str = "u.id AS user_id, u.name AS user_name, u.email AS user_email, \
                       r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
                       (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) AS role_user_count";

/// A ligação até os papéis.
///
/// O `deleted_at` do papel mora na condição do `JOIN`, não no `WHERE`. Movido
/// para o `WHERE`, ele descartaria a linha inteira quando não houvesse papel — o
/// `LEFT JOIN` viraria `INNER`, e um usuário sem papel nenhum sumiria da
/// listagem em vez de aparecer com a lista vazia.
const JOIN_ROLES: &str = "LEFT JOIN user_roles ur ON ur.user_id = u.id \
                          LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL";

/// Um usuário pelo id, com os papéis.
pub struct GetAccountDql {
    user_id: i64,
}

impl GetAccountDql {
    /// Monta a consulta.
    pub(crate) const fn new(user_id: i64) -> Self {
        Self { user_id }
    }
}

impl Dql for GetAccountDql {
    type View = Option<AccountView>;
}

impl SqlDql for GetAccountDql {
    fn build(&self) -> SqlQuery {
        Select::from("users u")
            .column(COLUMNS)
            .join(JOIN_ROLES)
            .filter("u.id = ?", [Bind::Int(self.user_id)])
            .filter("u.deleted_at IS NULL", [])
            .order_by("r.id ASC")
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        // Sem linha nenhuma o usuário não existe. Com linhas, ele existe mesmo
        // que todas tenham `role_id` nulo — é o usuário sem papel.
        let Some(first) = rows.first() else {
            return Ok(None);
        };

        Ok(Some(AccountView {
            id: Row::id(first, "user_id")?,
            name: Row::text(first, "user_name")?,
            email: Row::text(first, "user_email")?,
            roles: AccountReader::roles_of(&rows)?,
        }))
    }
}
