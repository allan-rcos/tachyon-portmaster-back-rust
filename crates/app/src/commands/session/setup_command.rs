//! O primeiro usuário do sistema.

/// Montar o sistema com o primeiro usuário.
#[derive(Debug, Clone)]
pub struct SetupCommand {
    /// Nome do administrador.
    pub name: String,
    /// E-mail do administrador.
    pub email: String,
    /// Senha do administrador.
    pub password: String,
}
