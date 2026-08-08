//! O que `PUT /roles/{id}/permissions` recebe.

/// O que descreve a troca de permissões de um papel.
#[derive(Debug, Clone, Default)]
pub(crate) struct RolePermissionsUpdateRequest {
    /// `permissions`.
    pub(crate) permissions: Option<Vec<String>>,
}
