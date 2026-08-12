//! Os testes de `user_tm_impl`.

use super::*;
use crate::security::intern::argon2_hasher::Argon2Hasher;
use crate::table_modules::intern::helpers::fields_of::fields_of;
use pretty_assertions::assert_eq;

/// Gerador determinístico: o teste não deve depender de relógio nem de sorte.
struct FixedIdGenerator;
impl DatabaseIdGenerator for FixedIdGenerator {
    fn next(&self) -> String {
        "1A2b3C".to_string()
    }
}

fn table_module() -> UserTMImpl<FixedIdGenerator, Argon2Hasher> {
    UserTMImpl::new(FixedIdGenerator, Argon2Hasher::new())
}

fn valid_user() -> Box<dyn User> {
    table_module()
        .create(
            "Ana".into(),
            "ana@portmaster.local".into(),
            "Portmaster1".into(),
            Vec::new(),
        )
        .expect("os dados do fixture são válidos")
}

#[test]
fn cria_usuario_valido_sem_guardar_a_senha() {
    let user = valid_user();

    assert_eq!(user.id(), "1A2b3C");
    assert_eq!(user.name(), "Ana");
    assert_eq!(user.email(), "ana@portmaster.local");
    assert_ne!(user.password_hash(), "Portmaster1");
    assert!(user.password_hash().starts_with("$argon2"));
    assert_eq!(user.deleted_at(), None);
}

/// O ponto do lote.
///
/// Quem enviou três campos errados descobre os três agora, não um por
/// requisição.
#[test]
fn acumula_todos_os_campos_invalidos_de_uma_vez() {
    let error = table_module()
        .create(
            String::new(),
            "sem-arroba".into(),
            "curta".into(),
            Vec::new(),
        )
        .err()
        .expect("nome vazio, e-mail inválido e senha fraca devem falhar");

    let UserError::Validation(fields) = error;
    assert_eq!(fields_of(&fields), vec!["name", "email", "password"]);
}

#[test]
fn recusa_senha_sem_a_variedade_exigida() {
    for weak in ["portmaster1", "PORTMASTER1", "PortmasterX", "Pm1"] {
        let error = table_module()
            .create("Ana".into(), "ana@x.com".into(), weak.into(), Vec::new())
            .err()
            .unwrap_or_else(|| panic!("{weak} deveria ser recusada"));

        let UserError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["password"], "senha: {weak}");
    }
}

#[test]
fn recusa_email_malformado() {
    for bad in [
        "sem-arroba",
        "@dominio.com",
        "ana@",
        "ana@dominio",
        "a b@c.com",
    ] {
        let error = table_module()
            .create("Ana".into(), bad.into(), "Portmaster1".into(), Vec::new())
            .err()
            .unwrap_or_else(|| panic!("{bad} deveria ser recusado"));

        let UserError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["email"], "e-mail: {bad}");
    }
}

/// Um nome só de espaço é um nome vazio.
///
/// É o que o `sanitize(trim)` garante — sem ele, o `not_empty` olharia para
/// os espaços e diria que está preenchido.
#[test]
fn nome_so_de_espaco_e_recusado_como_vazio() {
    let error = table_module()
        .create(
            "   ".into(),
            "ana@x.com".into(),
            "Portmaster1".into(),
            Vec::new(),
        )
        .err()
        .expect("nome em branco deve ser recusado");

    let UserError::Validation(fields) = error;
    assert_eq!(fields_of(&fields), vec!["name"]);
}

/// A transição produz um objeto novo, e não muta o argumento.
///
/// Se ela mutasse, uma atualização recusada mais adiante deixaria o
/// chamador com um objeto meio-alterado.
#[test]
fn update_nao_altera_o_usuario_recebido() {
    let original = valid_user();
    let updated = table_module()
        .update(
            original.as_ref(),
            "Ana Maria".into(),
            "ana.maria@x.com".into(),
        )
        .expect("os dados são válidos");

    assert_eq!(original.name(), "Ana");
    assert_eq!(updated.name(), "Ana Maria");
    assert_eq!(updated.id(), original.id());
}

#[test]
fn troca_de_senha_preserva_o_resto_e_muda_o_hash() {
    let original = valid_user();
    let changed = table_module()
        .change_password(original.as_ref(), "Portmaster2".into())
        .expect("a senha nova é válida");

    assert_eq!(changed.email(), original.email());
    assert_ne!(changed.password_hash(), original.password_hash());
}

#[test]
fn troca_de_senha_valida_a_nova() {
    let original = valid_user();
    let error = table_module()
        .change_password(original.as_ref(), "fraca".into())
        .err()
        .expect("senha fraca deve ser recusada também na troca");

    let UserError::Validation(fields) = error;
    assert_eq!(fields_of(&fields), vec!["password"]);
}
