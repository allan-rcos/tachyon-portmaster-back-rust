//! O contrato de persistência de container.

use portmaster_domain::domain::Container;

/// Persistência de contêineres.
#[trait_variant::make(Send)]
pub trait ContainerRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>>;

    /// Grava um contêiner novo.
    async fn insert(&self, container: &dyn Container) -> anyhow::Result<()>;

    /// Atualiza um contêiner existente.
    async fn update(&self, container: &dyn Container) -> anyhow::Result<()>;

    /// Remove um contêiner — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
