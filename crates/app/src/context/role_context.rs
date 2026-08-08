//! Um papel do usuário da requisição.

/// Um papel e o que ele concede.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleContext {
    /// Id em base62.
    pub id: String,
    /// Nome do papel.
    pub name: String,
    /// Os slugs de permissão concedidos.
    pub permissions: Vec<String>,
}

impl RoleContext {
    /// Se este papel concede a permissão.
    pub fn grants(&self, slug: &str) -> bool {
        self.permissions.iter().any(|granted| granted == slug)
    }
}
