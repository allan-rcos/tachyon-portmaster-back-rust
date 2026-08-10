//! O contrato de persistência de role.

use portmaster_domain::domain::Role;

/// Persistência de papéis.
#[trait_variant::make(Send)]
pub trait RoleRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>>;

    /// Os papéis de um usuário.
    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>>;

    /// Grava um papel novo.
    async fn insert(&self, role: &dyn Role) -> anyhow::Result<()>;

    /// Atualiza um papel existente.
    async fn update(&self, role: &dyn Role) -> anyhow::Result<()>;

    /// Remove um papel — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
