//! Os testes de `role_tm_impl`.

use super::*;
use crate::table_modules::intern::helpers::fields_of::fields_of;
use pretty_assertions::assert_eq;

#[derive(Clone)]
struct FixedIdGenerator;
impl DatabaseIdGenerator for FixedIdGenerator {
    fn next(&self) -> String {
        "9Z8y".to_string()
    }
}

fn table_module() -> impl RoleTM {
    role_tm(FixedIdGenerator)
}

#[test]
fn cria_papel_valido() {
    let role = table_module()
        .create("Operador".into(), vec!["container:read".into()])
        .expect("os dados são válidos");

    assert_eq!(role.id(), "9Z8y");
    assert_eq!(role.name(), "Operador");
    assert_eq!(role.permissions(), ["container:read"]);
}

#[test]
fn recusa_nome_vazio_ou_longo_demais() {
    for bad in ["", "   ", &"x".repeat(MAX_NAME_LENGTH + 1)] {
        let error = table_module()
            .create(bad.into(), Vec::new())
            .err()
            .expect("nome inválido deve falhar");

        let RoleError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["name"]);
    }
}

/// Um slug omitido é uma permissão **revogada**.
///
/// Se isto virasse merge, não haveria como tirar uma permissão de um papel.
#[test]
fn atualizar_permissoes_substitui_em_vez_de_somar() {
    let role = table_module()
        .create("Operador".into(), vec!["a:read".into(), "a:write".into()])
        .expect("os dados são válidos");

    let updated = table_module()
        .update_permissions(role.as_ref(), vec!["a:read".into()])
        .expect("a substituição não valida slugs");

    assert_eq!(updated.permissions(), ["a:read"]);
    assert_eq!(role.permissions(), ["a:read", "a:write"]);
}
