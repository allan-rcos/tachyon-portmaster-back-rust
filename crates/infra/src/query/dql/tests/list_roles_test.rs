//! Os testes de `list_roles`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_busca_entra_na_pagina_e_na_contagem() {
    let dql = ListRoles {
        limit: 20,
        search: Some("operador".into()),
        cursor: None,
    };

    assert_eq!(
        dql.build().sql(),
        "SELECT r.id, r.name, r.permissions, \
         (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count, \
         (SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total \
         FROM roles r WHERE r.id > ? AND r.deleted_at IS NULL AND r.search_name LIKE ? \
         ORDER BY r.id ASC LIMIT ?"
    );
}
