//! O read model `AccountView`.

use crate::query::views::RoleViewItem;
use serde::{Deserialize, Serialize};

/// Um usuário e os papéis dele.
///
/// Serve tanto `GET /account` (o próprio) quanto cada item de `GET /users` — é o
/// mesmo recorte, e duplicá-lo em dois tipos idênticos só criaria a chance de um
/// divergir do outro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountView {
    /// Id em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
    /// Os papéis atribuídos, ordenados por id.
    pub roles: Vec<RoleViewItem>,
}
