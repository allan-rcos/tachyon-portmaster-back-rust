//! Os testes de `get_product`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_busca_por_id_filtra_o_soft_delete() {
    let dql = GetProduct { id: 99 };

    assert_eq!(
        dql.build().sql(),
        "SELECT p.id, p.name, p.density, p.risk_class FROM products p \
         WHERE p.id = ? AND p.deleted_at IS NULL LIMIT 1"
    );
}

/// Uma URL inventada não deve abrir transação para consultar um número
/// arbitrário.
#[test]
fn id_fora_do_base62_e_recusado() {
    assert!(get_product("nao é base62!").is_err());
}
