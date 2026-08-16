//! A consulta de um usuário com os papéis dele.

use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::dql::list_roles::read_item as read_role;
use crate::query::views::{AccountView, RoleViewItem};
use crate::query::{Dql, SqlDql};

/// Um usuário pelo id, com os papéis.
pub fn get_account(id: &str) -> anyhow::Result<impl SqlDql<View = Option<AccountView>>> {
    Ok(GetAccount {
        user_id: Codec::decode_id(id)?,
    })
}

/// O papel de uma linha, ou `None` no lado vazio do `LEFT JOIN`.
///
/// `role_id` nulo é a marca de "este usuário não tem papel": o join não achou
/// par, e as demais colunas de papel vêm nulas junto.
pub(super) fn read_role_of(row: &Row) -> anyhow::Result<Option<RoleViewItem>> {
    let role_id: Option<i64> = Column::of(row, "role_id")?;

    if role_id.is_none() {
        return Ok(None);
    }

    read_role(row, "role_").map(Some)
}

/// Um usuário pelo id, com os papéis.
struct GetAccount {
    /// O alvo, como `BIGINT`.
    user_id: i64,
}

impl Dql for GetAccount {
    type View = Option<AccountView>;

    fn cache_key(&self) -> String {
        format!("get_account:{}", self.user_id)
    }
}

impl SqlDql for GetAccount {
    /// Usuário e papel na mesma linha, com o papel prefixado porque `id` e
    /// `name` existem dos dois lados.
    ///
    /// O `deleted_at` do papel mora na condição do `JOIN`, e não no `WHERE`.
    /// Movido para o `WHERE`, ele descartaria a linha inteira quando não
    /// houvesse papel — o `LEFT JOIN` viraria `INNER`, e um usuário sem papel
    /// nenhum sumiria da resposta em vez de aparecer com a lista vazia.
    fn build(&self) -> (String, Params) {
        let sql = "SELECT u.id AS user_id, u.name AS user_name, u.email AS user_email, \
             r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
             (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) \
              AS role_user_count \
             FROM users u \
             LEFT JOIN user_roles ur ON ur.user_id = u.id \
             LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL \
             WHERE u.id = :user_id AND u.deleted_at IS NULL \
             ORDER BY r.id ASC";

        (
            sql.to_owned(),
            vec![("user_id".to_owned(), Value::Int(self.user_id))].into(),
        )
    }

    /// Sem linha nenhuma o usuário não existe.
    ///
    /// Com linhas, ele existe mesmo que todas tenham `role_id` nulo — é o
    /// usuário sem papel.
    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let Some(first) = rows.first() else {
            return Ok(None);
        };

        let id: i64 = Column::of(first, "user_id")?;

        let mut roles = Vec::with_capacity(rows.len());
        for row in &rows {
            if let Some(role) = read_role_of(row)? {
                roles.push(role);
            }
        }

        Ok(Some(AccountView {
            id: Codec::encode_id(id),
            name: Column::of(first, "user_name")?,
            email: Column::of(first, "user_email")?,
            roles,
        }))
    }
}
