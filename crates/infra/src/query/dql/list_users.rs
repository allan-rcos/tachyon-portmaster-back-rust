//! A listagem paginada de usuários.

use anyhow::Context as _;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::entity::codec::Codec;
use crate::query::dql::get_account::{read_role_of, COLUMNS, JOIN_ROLES};
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
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(" FROM (SELECT id, name, email FROM users WHERE deleted_at IS NULL ORDER BY id ASC LIMIT ");
        builder.push_bind(i64::from(self.limit));
        builder.push(" OFFSET ");
        builder.push_bind(i64::from(self.offset));
        builder.push(") AS u ");
        builder.push(JOIN_ROLES);
        builder.push(" ORDER BY u.id ASC, r.id ASC");

        builder
    }

    /// Agrupa o fan-out de papéis de volta em um usuário por item.
    ///
    /// As linhas chegam agrupadas por usuário (`ORDER BY u.id`), então comparar
    /// com a última basta para saber se começou outro — sem mapa auxiliar e sem
    /// perder a ordem da consulta.
    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let mut items: Vec<AccountView> = Vec::with_capacity(self.limit as usize);

        for row in &rows {
            let raw: i64 = row
                .try_get("user_id")
                .context("coluna `user_id` não veio como inteiro")?;
            let id = Codec::encode_id(raw);

            if items.last().map(|last| last.id.as_str()) != Some(id.as_str()) {
                items.push(AccountView {
                    id,
                    name: row
                        .try_get("user_name")
                        .context("coluna `user_name` não veio como texto")?,
                    email: row
                        .try_get("user_email")
                        .context("coluna `user_email` não veio como texto")?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// É o que impede o fan-out de papéis de cortar a página no meio de um
    /// usuário.
    #[test]
    fn a_pagina_de_usuarios_sai_de_uma_tabela_derivada() {
        let dql = ListUsers {
            limit: 10,
            offset: 20,
        };

        assert_eq!(
            dql.build().sql(),
            "SELECT u.id AS user_id, u.name AS user_name, u.email AS user_email, \
             r.id AS role_id, r.name AS role_name, r.permissions AS role_permissions, \
             (SELECT COUNT(*) FROM user_roles urc WHERE urc.role_id = r.id) AS role_user_count \
             FROM (SELECT id, name, email FROM users WHERE deleted_at IS NULL \
             ORDER BY id ASC LIMIT ? OFFSET ?) AS u \
             LEFT JOIN user_roles ur ON ur.user_id = u.id \
             LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL \
             ORDER BY u.id ASC, r.id ASC"
        );
    }

    /// Pedir uma página além do fim é uma lista vazia, não um resultado
    /// sorteado: sem saturar, o produto estoura o `u32` e dá a volta.
    #[test]
    fn a_pagina_absurda_nao_da_a_volta() {
        assert_eq!(Paging::offset(Some(u32::MAX), 20), u32::MAX);
        assert_eq!(Paging::offset(None, 20), 0);
        assert_eq!(Paging::offset(Some(0), 20), 0);
        assert_eq!(Paging::offset(Some(3), 10), 20);
    }
}
