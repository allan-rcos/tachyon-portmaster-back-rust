//! Criar um usuário.

use crate::context::UserContext;

/// Cadastrar um usuário.
#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do usuário.
    pub name: String,
    /// E-mail — único no sistema.
    pub email: String,
    /// Senha inicial.
    pub initial_password: String,
    /// Ids dos papéis a atribuir, em base62.
    pub role_ids: Vec<String>,
}
