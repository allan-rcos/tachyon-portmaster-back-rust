//! A listagem paginada de usuários.

use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::dql::get_account::read_role_of;
use crate::query::dql::paging::Paging;
use crate::query::views::{AccountView, UserListView};
use crate::query::{Dql, SqlDql};

/// A listagem paginada de usuários.
///
/// Esta é a única listagem que pagina por **página**, e não por cursor: é o que
/// o contrato publicado já expõe.
pub fn list_users(page: Option<u32>, limit: Option<u32>) -> impl SqlDql<View = UserListView> {
    let limit = Paging::effective_limit(limit);

    ListUsers {
        limit,
        offset: Paging::offset(page, limit),
    }
}

/// A listagem de usuários.
struct ListUsers {
    /// O tamanho da página, já resolvido.
    limit: u32,
    /// Quantos usuários pular.
    offset: u32,
}

impl Dql for ListUsers {
    type View = UserListView;

    fn cache_key(&self) -> String {
        format!("list_users:{}:{}", self.limit, self.offset)
    }
}

impl SqlDql for ListUsers {
    /// A página sai numa tabela derivada, e não de um `LIMIT` no `SELECT` de
    /// fora.
    ///
    /// O `LEFT JOIN` com papéis multiplica as linhas por usuário, então um
    /// `LIMIT` externo cortaria no meio de um usuário: o vigésimo apareceria com
    /// parte dos papéis e nenhum indício de que faltou.
    fn build(&self) -> (String, Params) {
        let sql = "SELECT u.id AS user_id, u.name AS user_name, u.email AS user_email, \
             r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
             (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) \
              AS role_user_count \
             FROM (SELECT id, name, email FROM users WHERE deleted_at IS NULL \
                   ORDER BY id ASC LIMIT :limit OFFSET :offset) AS u \
             LEFT JOIN user_roles ur ON ur.user_id = u.id \
             LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL \
             ORDER BY u.id ASC, r.id ASC";

        let values = vec![
            ("limit".to_owned(), Value::Int(i64::from(self.limit))),
            ("offset".to_owned(), Value::Int(i64::from(self.offset))),
        ];

        (sql.to_owned(), values.into())
    }

    /// Agrupa o fan-out de papéis de volta em um usuário por item.
    ///
    /// As linhas chegam agrupadas por usuário (`ORDER BY u.id`), então comparar
    /// com a última basta para saber se começou outro — sem mapa auxiliar e sem
    /// perder a ordem da consulta.
    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let mut items: Vec<AccountView> = Vec::with_capacity(self.limit as usize);

        for row in &rows {
            let raw: i64 = Column::of(row, "user_id")?;
            let id = Codec::encode_id(raw);

            if items.last().map(|last| last.id.as_str()) != Some(id.as_str()) {
                items.push(AccountView {
                    id,
                    name: Column::of(row, "user_name")?,
                    email: Column::of(row, "user_email")?,
                    roles: Vec::new(),
                });
            }

            if let Some(role) = read_role_of(row)? {
                // `last` existe: ou já estava, ou acabou de ser inserido acima.
                if let Some(user) = items.last_mut() {
                    user.roles.push(role);
                }
            }
        }

        Ok(UserListView { items })
    }
}
