//! O contrato de persistência de product.

use portmaster_domain::domain::Product;

/// Persistência de produtos.
#[trait_variant::make(Send)]
pub trait ProductRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>>;

    /// Grava um produto novo.
    async fn insert(&self, product: &dyn Product) -> anyhow::Result<()>;

    /// Atualiza um produto existente.
    async fn update(&self, product: &dyn Product) -> anyhow::Result<()>;

    /// Remove um produto — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
