//! Os testes de `marker_tm_impl`.

use super::*;
use crate::security::SecurityProvider;
use pretty_assertions::assert_eq;

fn table_module() -> impl MarkerTM {
    marker_tm(SecurityProvider::index())
}

#[test]
fn guarda_o_digest_e_nao_o_valor() {
    let marker = table_module()
        .create("refresh-token".into(), "token-secreto", true)
        .expect("grupo e valor são válidos");

    assert_eq!(marker.group(), "refresh-token");
    assert_ne!(marker.key(), "token-secreto");
    assert!(marker.flag());
}

/// É o que faz o refresh funcionar.
///
/// Marcar no login e consultar depois precisam cair na mesma chave.
#[test]
fn o_mesmo_valor_reencontra_a_mesma_marca() {
    let first = table_module()
        .create("refresh-token".into(), "abc", true)
        .expect("válido");
    let second = table_module()
        .create("refresh-token".into(), "abc", false)
        .expect("válido");

    assert_eq!(first.key(), second.key());
}

#[test]
fn recusa_valor_vazio() {
    let error = table_module()
        .create("refresh-token".into(), "", true)
        .err()
        .expect("valor vazio colidiria todo mundo numa marca só");

    let MarkerError::Validation(fields) = error;
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].field, "value");
}

#[test]
fn recusa_grupo_fora_do_formato() {
    for bad in ["", "Refresh-Token", "refresh_token"] {
        assert!(
            table_module().create(bad.into(), "abc", true).is_err(),
            "deveria recusar grupo {bad:?}"
        );
    }
}
