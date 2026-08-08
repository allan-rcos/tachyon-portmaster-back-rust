//! Produtos.

use crate::commands::product::CreateProductCommand;
use crate::commands::product::DeleteProductCommand;
use crate::commands::product::UpdateProductCommand;
use crate::error::AppError;
use crate::queries::product::GetProductQuery;
use crate::queries::product::ListProductsQuery;
use portmaster_domain::models::Product;
use portmaster_infra::query::views::{ProductListView, ProductViewItem};

/// O que a apresentação pode pedir sobre produtos.
#[trait_variant::make(Send)]
pub trait ProductUseCase {
    /// Cadastra e devolve o produto criado.
    async fn create(&self, command: CreateProductCommand) -> Result<Box<dyn Product>, AppError>;

    /// Altera e devolve o produto atualizado.
    async fn update(&self, command: UpdateProductCommand) -> Result<Box<dyn Product>, AppError>;

    /// Remove — soft-delete, porque o manifesto histórico referencia o produto.
    async fn delete(&self, command: DeleteProductCommand) -> Result<(), AppError>;

    /// Lê um produto.
    async fn get(&self, query: GetProductQuery) -> Result<ProductViewItem, AppError>;

    /// Lista produtos.
    async fn list(&self, query: ListProductsQuery) -> Result<ProductListView, AppError>;
}
