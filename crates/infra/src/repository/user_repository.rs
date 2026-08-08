//! O contrato de persistência de user.

use portmaster_domain::models::User;

/// Persistência de usuários.
#[trait_variant::make(Send)]
pub trait UserRepository {
    /// Se existe ao menos um usuário.
    ///
    /// É o que o bootstrap consulta: um sistema sem usuário nenhum aceita o
    /// primeiro cadastro; com um, recusa.
    async fn has_any(&self) -> anyhow::Result<bool>;

    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn User>>>;

    /// Busca por e-mail, ou `None` se não existe ou foi removido.
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Box<dyn User>>>;

    /// Grava um usuário novo.
    async fn insert(&self, user: &dyn User) -> anyhow::Result<()>;

    /// Atualiza um usuário existente.
    async fn update(&self, user: &dyn User) -> anyhow::Result<()>;

    /// Remove um usuário — soft-delete, porque usuário é entidade forte.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;

    /// Substitui os papéis de um usuário.
    ///
    /// Substitui, não soma: um papel omitido está revogado.
    async fn sync_roles(&self, user_id: &str, role_ids: &[String]) -> anyhow::Result<()>;
}
