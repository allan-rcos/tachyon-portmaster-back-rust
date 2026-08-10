//! As regras de papel.

use crate::domain::Role;
use crate::error::RoleError;

/// Constrói e altera papéis.
pub trait RoleTM {
    /// Cria um papel novo, ainda não persistido.
    fn create(&self, name: String, permissions: Vec<String>) -> Result<Box<dyn Role>, RoleError>;

    /// Produz o papel com outro conjunto de permissões.
    ///
    /// A lista **substitui** a anterior: o que não vier nela está revogado.
    fn update_permissions(
        &self,
        role: &dyn Role,
        permissions: Vec<String>,
    ) -> Result<Box<dyn Role>, RoleError>;
}
