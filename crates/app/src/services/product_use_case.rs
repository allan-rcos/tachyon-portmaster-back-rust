//! Produtos.

use crate::commands::product::CreateProductCommand;
use crate::commands::product::DeleteProductCommand;
use crate::commands::product::UpdateProductCommand;
use crate::error::ProductError;
use crate::queries::product::GetProductQuery;
use crate::queries::product::ListProductsQuery;
use crate::services::MetadataUseCase;
use portmaster_domain::domain::Product;
use portmaster_infra::query::views::{ProductListView, ProductViewItem};

/// O que a apresentação pode pedir sobre produtos.
#[trait_variant::make(Send)]
pub trait ProductUseCase {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataUseCase + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ProductError>;

    /// Cadastra e devolve o produto criado.
    async fn create(&self, command: CreateProductCommand)
        -> Result<Box<dyn Product>, ProductError>;

    /// Altera e devolve o produto atualizado.
    async fn update(&self, command: UpdateProductCommand)
        -> Result<Box<dyn Product>, ProductError>;

    /// Remove — soft-delete, porque o manifesto histórico referencia o produto.
    async fn delete(&self, command: DeleteProductCommand) -> Result<(), ProductError>;

    /// Lê um produto.
    async fn get(&self, query: GetProductQuery) -> Result<ProductViewItem, ProductError>;

    /// Lista produtos.
    async fn list(&self, query: ListProductsQuery) -> Result<ProductListView, ProductError>;
}
