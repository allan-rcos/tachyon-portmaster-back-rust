//! Os testes de `list_users`.

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
