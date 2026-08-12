//! Os testes de `user_context`.

use super::*;

fn role(name: &str, permissions: &[&str]) -> RoleContext {
    RoleContext {
        id: "1".into(),
        name: name.into(),
        permissions: permissions.iter().map(|p| (*p).to_owned()).collect(),
    }
}

fn user(roles: Vec<RoleContext>) -> UserContext {
    UserContext {
        id: "1".into(),
        name: "Ana".into(),
        email: "ana@portmaster.local".into(),
        roles,
    }
}

/// Papéis somam.
///
/// Exigir que todos concedam faria acrescentar um papel **reduzir** o que
/// o usuário pode fazer.
#[test]
fn qualquer_papel_basta_para_conceder() {
    let context = user(vec![
        role("Leitor", &["product:read"]),
        role("Operador", &["container:seal"]),
    ]);

    assert!(context.has_permission("product:read"));
    assert!(context.has_permission("container:seal"));
}

#[test]
fn sem_papel_nenhum_nao_ha_permissao() {
    assert!(!user(Vec::new()).has_permission("product:read"));
}

/// `product:read` não pode ser satisfeita por `product:read-all` nem por
/// um prefixo — permissão não tem hierarquia neste sistema.
#[test]
fn a_comparacao_de_slug_e_exata() {
    let context = user(vec![role("Quase", &["product:read-all"])]);

    assert!(!context.has_permission("product:read"));
}
