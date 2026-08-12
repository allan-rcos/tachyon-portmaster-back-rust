//! Os testes de `permission_tm_impl`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn aceita_os_slugs_que_os_casos_de_uso_declaram() {
    for good in [
        "product:create",
        "container:seal",
        "manifest:load",
        "user:update-roles",
        "role:update-permissions",
        "metrics:read",
    ] {
        let permission = PermissionTMImpl::new()
            .create(good.into())
            .unwrap_or_else(|_| panic!("{good} deveria ser aceito"));
        assert_eq!(permission.slug(), good);
    }
}

#[test]
fn recusa_slug_fora_do_formato() {
    for bad in [
        "",
        "sem-dois-pontos",
        "product:",
        ":create",
        "Product:Create",
        "product:create:extra",
        "product::create",
    ] {
        assert!(
            PermissionTMImpl::new().create(bad.into()).is_err(),
            "deveria recusar {bad:?}"
        );
    }
}

#[test]
fn recusa_slug_longo_demais() {
    let long = format!("{}:{}", "a".repeat(40), "b".repeat(40));
    assert!(PermissionTMImpl::new().create(long).is_err());
}
