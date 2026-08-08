//! O que `POST /roles` recebe.

/// O que descreve a criação de um papel.
#[derive(Debug, Clone, Default)]
pub(crate) struct RoleCreateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `permissions`.
    pub(crate) permissions: Option<Vec<String>>,
}
