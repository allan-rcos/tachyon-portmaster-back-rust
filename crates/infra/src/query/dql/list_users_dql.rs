//! A listagem paginada de usuários.

use crate::query::dql::account_reader::AccountReader;
use crate::query::dql::paging::Paging;
use crate::query::params::UserListParams;
use crate::query::row::Row;
use crate::query::sql::{Select, SqlQuery};
use crate::query::views::{AccountView, UserListView};
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

/// A listagem de usuários.
pub struct ListUsersDql {
    params: UserListParams,
}

impl ListUsersDql {
    /// Monta a consulta.
    pub(crate) const fn new(params: UserListParams) -> Self {
        Self { params }
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        Paging::effective_limit(self.params.limit)
    }

    /// Quantos usuários pular. Página ausente ou zero é a primeira.
    ///
    /// O `page` vem da query string, então é um `u32` arbitrário que o cliente
    /// escolhe. Multiplicá-lo pelo limite estoura o `u32` a partir de ~86
    /// milhões de páginas — em release isso daria a volta e devolveria a página
    /// errada em silêncio. Saturar é o comportamento certo: pedir uma página
    /// além do fim é uma lista vazia, não um resultado sorteado.
    fn offset(&self) -> u32 {
        let page = self.params.page.filter(|value| *value > 0).unwrap_or(1);

        page.saturating_sub(1).saturating_mul(self.limit())
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
            let id = Row::id(row, "user_id")?;

            // As linhas chegam agrupadas por usuário (ORDER BY u.id), então
            // comparar com a última basta para saber se começou outro — sem
            // mapa auxiliar e sem perder a ordem da consulta.
            if items.last().map(|last| last.id.as_str()) != Some(id.as_str()) {
                items.push(AccountView {
                    id,
                    name: Row::text(row, "user_name")?,
                    email: Row::text(row, "user_email")?,
                    roles: Vec::new(),
                });
            }

            if let Some(role) = AccountReader::role_of(row)? {
                // `last` existe: ou já estava, ou acabou de ser inserido acima.
                if let Some(user) = items.last_mut() {
                    user.roles.push(role);
                }
            }
        }

        Ok(UserListView { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::dql::get_account_dql::GetAccountDql;
    use crate::query::sql::Bind;
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
