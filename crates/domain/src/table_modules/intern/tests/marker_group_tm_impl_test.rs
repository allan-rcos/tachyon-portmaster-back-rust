//! Os testes de `marker_group_tm_impl`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn aceita_o_grupo_da_sessao_de_refresh() {
    let group = MarkerGroupTMImpl::new()
        .create("refresh-token".into())
        .expect("o grupo da sessão é válido");

    assert_eq!(group.slug(), "refresh-token");
}

#[test]
fn recusa_slug_fora_do_formato() {
    for bad in ["", "Refresh-Token", "refresh_token", "-refresh", "refresh-"] {
        assert!(
            MarkerGroupTMImpl::new().create(bad.into()).is_err(),
            "deveria recusar {bad:?}"
        );
    }
}
