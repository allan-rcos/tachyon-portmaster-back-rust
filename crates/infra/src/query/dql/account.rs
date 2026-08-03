//! As consultas de conta — um usuário e os papéis dele.

use sqlx::mysql::MySqlRow;

use super::{effective_limit, role};
use crate::query::row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{AccountView, RoleViewItem, UserListView};
use crate::query::{Dql, SqlDql, UserListParams};

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
pub(crate) struct GetAccountDql {
    user_id: i64,
}

impl GetAccountDql {
    /// Monta a consulta.
    pub(crate) fn new(user_id: i64) -> Self {
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
            id: row::id(first, "user_id")?,
            name: row::text(first, "user_name")?,
            email: row::text(first, "user_email")?,
            roles: roles_of(&rows)?,
        }))
    }
}

/// A listagem de usuários.
pub(crate) struct ListUsersDql {
    params: UserListParams,
}

impl ListUsersDql {
    /// Monta a consulta.
    pub(crate) fn new(params: UserListParams) -> Self {
        Self { params }
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        effective_limit(self.params.limit)
    }

    /// Quantos usuários pular. Página ausente ou zero é a primeira.
    fn offset(&self) -> u32 {
        let page = self.params.page.filter(|value| *value > 0).unwrap_or(1);

        (page - 1) * self.limit()
    }
}

impl Dql for ListUsersDql {
    type View = UserListView;
}

impl SqlDql for ListUsersDql {
    fn build(&self) -> SqlQuery {
        // A página sai numa tabela derivada, e não de um LIMIT no SELECT de
        // fora. O `LEFT JOIN` com papéis multiplica as linhas por usuário, então
        // um LIMIT externo cortaria no meio de um usuário: o vigésimo apareceria
        // com parte dos papéis e nenhum indício de que faltou.
        let page = Select::from("users")
            .column("id, name, email")
            .filter("deleted_at IS NULL", [])
            .order_by("id ASC")
            .limit(self.limit())
            .offset(self.offset())
            .to_sql();

        Select::from(format!("({page}) AS u"))
            .column(COLUMNS)
            .join(JOIN_ROLES)
            .order_by("u.id ASC, r.id ASC")
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let mut items: Vec<AccountView> = Vec::with_capacity(self.limit() as usize);

        for row in &rows {
            let id = row::id(row, "user_id")?;

            // As linhas chegam agrupadas por usuário (ORDER BY u.id), então
            // comparar com a última basta para saber se começou outro — sem
            // mapa auxiliar e sem perder a ordem da consulta.
            if items.last().map(|last| last.id.as_str()) != Some(id.as_str()) {
                items.push(AccountView {
                    id,
                    name: row::text(row, "user_name")?,
                    email: row::text(row, "user_email")?,
                    roles: Vec::new(),
                });
            }

            if let Some(role) = role_of(row)? {
                // `last` existe: ou já estava, ou acabou de ser inserido acima.
                if let Some(user) = items.last_mut() {
                    user.roles.push(role);
                }
            }
        }

        Ok(UserListView { items })
    }
}

/// Os papéis presentes num conjunto de linhas do mesmo usuário.
fn roles_of(rows: &[MySqlRow]) -> anyhow::Result<Vec<RoleViewItem>> {
    let mut roles = Vec::with_capacity(rows.len());

    for row in rows {
        if let Some(role) = role_of(row)? {
            roles.push(role);
        }
    }

    Ok(roles)
}

/// O papel de uma linha, ou `None` no lado vazio do `LEFT JOIN`.
fn role_of(row: &MySqlRow) -> anyhow::Result<Option<RoleViewItem>> {
    // `role_id` nulo é a marca de "este usuário não tem papel": o join não achou
    // par, e as demais colunas de papel vêm nulas junto.
    if row::opt_id(row, "role_id")?.is_none() {
        return Ok(None);
    }

    role::item_prefixed(row, "role_").map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_conta_traz_os_papeis_por_left_join() {
        let query = GetAccountDql::new(123).build();

        assert!(
            query
                .sql
                .contains("LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL"),
            "o filtro de papel apagado tem que ficar no JOIN: {}",
            query.sql
        );
        assert_eq!(query.binds, vec![Bind::Int(123)]);
    }

    #[test]
    fn a_pagina_de_usuarios_sai_de_uma_tabela_derivada() {
        // É o que impede o fan-out de papéis de cortar a página no meio de um
        // usuário.
        let query = ListUsersDql::new(UserListParams {
            page: Some(3),
            limit: Some(10),
        })
        .build();

        assert_eq!(
            query.sql,
            "SELECT u.id AS user_id, u.name AS user_name, u.email AS user_email, \
             r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
             (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) AS role_user_count \
             FROM (SELECT id, name, email FROM users WHERE deleted_at IS NULL \
             ORDER BY id ASC LIMIT 10 OFFSET 20) AS u \
             LEFT JOIN user_roles ur ON ur.user_id = u.id \
             LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL \
             ORDER BY u.id ASC, r.id ASC"
        );
        assert!(query.binds.is_empty());
    }

    #[test]
    fn a_primeira_pagina_nao_pula_nada() {
        for page in [None, Some(0), Some(1)] {
            let dql = ListUsersDql::new(UserListParams { page, limit: None });

            assert_eq!(dql.offset(), 0, "página {page:?} não deveria pular linhas");
        }
    }
}
