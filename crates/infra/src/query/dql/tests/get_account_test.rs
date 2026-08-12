//! Os testes de `get_account`.

use super::*;

#[test]
fn a_conta_traz_os_papeis_por_left_join() {
    let dql = GetAccount { user_id: 123 };
    let sql = dql.build().sql().as_str().to_owned();

    assert!(
        sql.contains("LEFT JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL"),
        "o filtro de papel apagado tem que ficar no JOIN: {sql}"
    );
}
