//! A consulta de um usuário com os papéis dele.

use anyhow::Context as _;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::entity::codec::Codec;
use crate::query::dql::list_roles::read_item as read_role;
use crate::query::views::{AccountView, RoleViewItem};
use crate::query::{Dql, SqlDql};

/// A projeção comum às duas consultas de conta.
///
/// Usuário e papel na mesma linha, com o papel prefixado porque `id` e `name`
/// existem dos dois lados.
pub(super) const COLUMNS: &str = "u.id AS user_id, u.name AS user_name, u.email AS user_email, \
     r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
     (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) AS role_user_count";

/// A ligação até os papéis.
///
/// O `deleted_at` do papel mora na condição do `JOIN`, não no `WHERE`. Movido
/// para o `WHERE`, ele descartaria a linha inteira quando não houvesse papel — o
/// `LEFT JOIN` viraria `INNER`, e um usuário sem papel nenhum sumiria da
/// listagem em vez de aparecer com a lista vazia.
pub(super) const JOIN_ROLES: &str = "LEFT JOIN user_roles ur ON ur.user_id = u.id \
                                     LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL";

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
pub(super) fn read_role_of(row: &MySqlRow) -> anyhow::Result<Option<RoleViewItem>> {
    let role_id: Option<i64> = row
        .try_get("role_id")
        .context("coluna `role_id` não veio como inteiro opcional")?;

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
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(" FROM users u ");
        builder.push(JOIN_ROLES);
        builder.push(" WHERE u.id = ");
        builder.push_bind(self.user_id);
        builder.push(" AND u.deleted_at IS NULL ORDER BY r.id ASC");

        builder
    }

    /// Sem linha nenhuma o usuário não existe.
    ///
    /// Com linhas, ele existe mesmo que todas tenham `role_id` nulo — é o
    /// usuário sem papel.
    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let Some(first) = rows.first() else {
            return Ok(None);
        };

        let id: i64 = first
            .try_get("user_id")
            .context("coluna `user_id` não veio como inteiro")?;

        let mut roles = Vec::with_capacity(rows.len());
        for row in &rows {
            if let Some(role) = read_role_of(row)? {
                roles.push(role);
            }
        }

        Ok(Some(AccountView {
            id: Codec::encode_id(id),
            name: first
                .try_get("user_name")
                .context("coluna `user_name` não veio como texto")?,
            email: first
                .try_get("user_email")
                .context("coluna `user_email` não veio como texto")?,
            roles,
        }))
    }
}

#[cfg(test)]
#[path = "tests/get_account_test.rs"]
mod tests;
