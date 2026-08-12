//! Os testes de `get_container`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_busca_por_id_filtra_o_soft_delete() {
    let dql = GetContainer { id: 42 };

    assert_eq!(
        dql.build().sql(),
        "SELECT c.id, c.code, c.current_weight, c.max_capacity, c.status \
         FROM containers c WHERE c.id = ? AND c.deleted_at IS NULL LIMIT 1"
    );
}
