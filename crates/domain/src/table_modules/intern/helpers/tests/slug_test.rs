//! Os testes de `slug`.

use super::*;

#[test]
fn aceita_tokens_kebab_validos() {
    for good in ["metrics", "refresh-token", "rate-limit", "a1", "x-9-y"] {
        assert!(Slug::try_new(good).is_ok(), "deveria aceitar {good}");
    }
}

#[test]
fn recusa_o_que_nao_e_kebab() {
    for bad in [
        "",
        "-leading",
        "trailing-",
        "double--hyphen",
        "Upper",
        "9leading",
        "with_underscore",
        "with space",
    ] {
        assert!(Slug::try_new(bad).is_err(), "deveria recusar {bad:?}");
    }
}

#[test]
fn recusa_token_longo_demais() {
    let long = "a".repeat(Slug::MAX_LENGTH.saturating_add(1));

    assert!(matches!(
        Slug::try_new(long),
        Err(SlugError::LenCharMaxViolated)
    ));
}
