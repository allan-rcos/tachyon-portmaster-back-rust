//! Os testes de `get_role`.

use super::*;
use pretty_assertions::assert_eq;

/// Com `LEFT JOIN` + `GROUP BY`, um papel sem nenhum usuário sairia da
/// listagem em vez de sair com zero.
#[test]
fn a_contagem_de_usuarios_e_correlacionada() {
    let dql = GetRole { id: 7 };

    assert_eq!(
        dql.build().sql(),
        "SELECT r.id, r.name, r.permissions, \
         (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id) AS user_count \
         FROM roles r WHERE r.id = ? AND r.deleted_at IS NULL LIMIT 1"
    );
}
