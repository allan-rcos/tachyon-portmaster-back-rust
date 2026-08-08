//! As regras de usuário.

use crate::error::UserError;
use crate::models::{Role, User};

/// Constrói e altera usuários, recusando-se a produzir um inválido.
///
/// Recebe **valores soltos**, nunca um `Command`: Command é vocabulário do
/// `app`, e o núcleo não o conhece.
pub trait UserTM {
    /// Cria um usuário novo, ainda não persistido.
    fn create(
        &self,
        name: String,
        email: String,
        password: String,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outro nome e e-mail.
    fn update(
        &self,
        user: &dyn User,
        name: String,
        email: String,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outra senha.
    ///
    /// Vale tanto para a troca feita pelo próprio dono quanto para a
    /// redefinição feita por um administrador: a regra sobre o que é uma senha
    /// aceitável é a mesma, e quem pode fazer o quê é decisão do `app`.
    fn change_password(
        &self,
        user: &dyn User,
        new_password: String,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outro conjunto de papéis.
    fn update_roles(
        &self,
        user: &dyn User,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError>;
}
