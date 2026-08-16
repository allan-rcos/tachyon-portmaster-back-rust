//! Quem está fazendo a requisição.

use crate::context::RoleContext;

/// O usuário da requisição, como o token o descreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserContext {
    /// Id em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
    /// Os papéis que ele carrega.
    pub roles: Vec<RoleContext>,
}

impl UserContext {
    /// Se algum papel concede a permissão.
    ///
    /// Basta **um**: papéis somam permissões, não as restringem.
    pub fn has_permission(&self, slug: &str) -> bool {
        self.roles.iter().any(|role| role.grants(slug))
    }
}
