//! O read model `RoleViewItem`.

use serde::{Deserialize, Serialize};

/// Um papel e o tamanho da sua população.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleViewItem {
    /// Id em base62.
    pub id: String,
    /// Nome do papel.
    pub name: String,
    /// Quantos usuários o têm — computado, sem par na tabela.
    pub user_count: i64,
    /// Os slugs de permissão que ele concede.
    pub permissions: Vec<String>,
}
