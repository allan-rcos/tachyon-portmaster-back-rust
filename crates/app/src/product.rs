//! Os casos de uso de produto.
//!
//! Este módulo é o molde dos demais: mostra o esqueleto de **escrita**
//! (autoriza → transação → TableModule → repositório → invalida cache) e o de
//! **leitura** (autoriza → cache → transação → DQL → executa), que todos os
//! outros repetem.

use portmaster_domain::enums::RiskClass;
use portmaster_domain::product::{Product, ProductTM};
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::{ProductListView, ProductViewItem};
use portmaster_infra::query::{ListParams, QueryFactory, QueryRepository};
use portmaster_infra::repository::ProductRepository;

use crate::authorization::{slug, RequiresPermission};
use crate::cache::{self, prefix};
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Cadastrar um produto.
#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do produto.
    pub name: String,
    /// Densidade, para converter quantidade em peso.
    pub density: f64,
    /// Classe de risco.
    pub risk_class: RiskClass,
}

/// Alterar um produto.
#[derive(Debug, Clone)]
pub struct UpdateProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
    /// Nome do produto.
    pub name: String,
    /// Densidade.
    pub density: f64,
    /// Classe de risco.
    pub risk_class: RiskClass,
}

/// Remover um produto.
#[derive(Debug, Clone)]
pub struct DeleteProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
}

/// Ler um produto.
#[derive(Debug, Clone)]
pub struct GetProductQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
}

/// Listar produtos.
#[derive(Debug, Clone)]
pub struct ListProductsQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
    /// Termo de busca.
    pub search: Option<String>,
}

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

/// A implementação, genérica sobre os ports que consome.
///
/// Nada de `Arc<dyn>`: os tipos concretos chegam do provider e o compilador
/// monomorfiza o grafo inteiro. Um caso de uso que não pudesse ser montado seria
/// erro de compilação, não surpresa no primeiro request.
pub(crate) struct ProductUseCaseImpl<R, T, Q, F, C, U> {
    products: R,
    product_tm: T,
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
    create_permission: RequiresPermission,
    update_permission: RequiresPermission,
    delete_permission: RequiresPermission,
    read_permission: RequiresPermission,
}

impl<R, T, Q, F, C, U> ProductUseCaseImpl<R, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) fn new(
        products: R,
        product_tm: T,
        queries: Q,
        dqls: F,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            products,
            product_tm,
            queries,
            dqls,
            cache,
            unit_of_work,
            create_permission: RequiresPermission::new(slug::PRODUCT_CREATE),
            update_permission: RequiresPermission::new(slug::PRODUCT_UPDATE),
            delete_permission: RequiresPermission::new(slug::PRODUCT_DELETE),
            read_permission: RequiresPermission::new(slug::PRODUCT_READ),
        }
    }
}

impl<R, T, Q, F, C, U> ProductUseCase for ProductUseCaseImpl<R, T, Q, F, C, U>
where
    R: ProductRepository + Send + Sync,
    T: ProductTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create(&self, command: CreateProductCommand) -> Result<Box<dyn Product>, AppError> {
        self.create_permission.authorize(&command.context)?;

        let product = transaction(&self.unit_of_work, async {
            // O TableModule valida e instancia — o caso de uso nunca constrói um
            // objeto de domínio, senão a regra de validação teria dois donos.
            let product =
                self.product_tm
                    .create(command.name, command.density, command.risk_class)?;

            self.products.insert(product.as_ref()).await?;

            Ok(product)
        })
        .await?;

        cache::invalidate(&self.cache, cache::PRODUCT_WRITE).await?;

        Ok(product)
    }

    async fn update(&self, command: UpdateProductCommand) -> Result<Box<dyn Product>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let product = transaction(&self.unit_of_work, async {
            let existing = self
                .products
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("produto", &command.id))?;

            let updated = self.product_tm.update(
                existing.as_ref(),
                command.name,
                command.density,
                command.risk_class,
            )?;

            self.products.update(updated.as_ref()).await?;

            Ok(updated)
        })
        .await?;

        cache::invalidate(&self.cache, cache::PRODUCT_WRITE).await?;

        Ok(product)
    }

    async fn delete(&self, command: DeleteProductCommand) -> Result<(), AppError> {
        self.delete_permission.authorize(&command.context)?;

        transaction(&self.unit_of_work, async {
            // Confere a existência antes de apagar: sem isso, remover duas vezes
            // responderia sucesso na segunda, e o cliente não teria como saber
            // que o id que ele mandou nunca existiu.
            self.products
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("produto", &command.id))?;

            self.products.delete(&command.id).await?;

            Ok(())
        })
        .await?;

        cache::invalidate(&self.cache, cache::PRODUCT_WRITE).await?;

        Ok(())
    }

    async fn get(&self, query: GetProductQuery) -> Result<ProductViewItem, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = cache::key(prefix::PRODUCT, "get", &[&query.id]);

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.get_product(&query.id)?;

            transaction(&self.unit_of_work, async {
                self.queries
                    .run(dql)
                    .await?
                    .ok_or_else(|| AppError::not_found("produto", &query.id))
            })
            .await
        })
        .await
    }

    async fn list(&self, query: ListProductsQuery) -> Result<ProductListView, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = cache::key(
            prefix::PRODUCT,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.search.as_deref().unwrap_or_default(),
            ],
        );

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.list_products(ListParams {
                cursor: query.cursor.clone(),
                limit: query.limit,
                search: query.search.clone(),
            });

            transaction(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
