//! A orquestração de produtos.

use crate::cache::cache_key::CacheKey;
use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::product::CreateProductCommand;
use crate::commands::product::DeleteProductCommand;
use crate::commands::product::UpdateProductCommand;
use crate::error::AppError;
use crate::queries::product::GetProductQuery;
use crate::queries::product::ListProductsQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::ProductUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::Product;
use portmaster_domain::table_modules::ProductTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::params::ListParams;
use portmaster_infra::query::views::{ProductListView, ProductViewItem};
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::ProductRepository;

/// A implementação, genérica sobre os ports que consome.
///
/// Nada de `Arc<dyn>`: os tipos concretos chegam do provider e o compilador
/// monomorfiza o grafo inteiro. Um caso de uso que não pudesse ser montado seria
/// erro de compilação, não surpresa no primeiro request.
pub(crate) struct ProductUseCaseImpl<R, T, Q, F, C, U> {
    /// Persistência de produtos.
    products: R,
    /// As regras de produto.
    product_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// De onde os DQLs saem, já com os parâmetros.
    dqls: F,
    /// O cache de leitura, para o read-through e a invalidação.
    cache: C,
    /// Quem abre e fecha a transação.
    unit_of_work: U,
    /// A permissão exigida para create.
    create_permission: RequiresPermission,
    /// A permissão exigida para update.
    update_permission: RequiresPermission,
    /// A permissão exigida para delete.
    delete_permission: RequiresPermission,
    /// A permissão exigida para read.
    read_permission: RequiresPermission,
}

impl<R, T, Q, F, C, U> ProductUseCaseImpl<R, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) const fn new(
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
            create_permission: RequiresPermission::new(PermissionSlug::PRODUCT_CREATE),
            update_permission: RequiresPermission::new(PermissionSlug::PRODUCT_UPDATE),
            delete_permission: RequiresPermission::new(PermissionSlug::PRODUCT_DELETE),
            read_permission: RequiresPermission::new(PermissionSlug::PRODUCT_READ),
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
    /// Cria um produto.
    ///
    /// O `TableModule` valida e instancia — o caso de uso **nunca** constrói um
    /// objeto de domínio, senão a regra de validação teria dois donos.
    async fn create(&self, command: CreateProductCommand) -> Result<Box<dyn Product>, AppError> {
        self.create_permission.authorize(&command.context)?;

        let product = Transaction::run(&self.unit_of_work, async {
            let product =
                self.product_tm
                    .create(command.name, command.density, command.risk_class)?;

            self.products.insert(product.as_ref()).await?;

            Ok(product)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::PRODUCT_WRITE).await?;

        Ok(product)
    }

    async fn update(&self, command: UpdateProductCommand) -> Result<Box<dyn Product>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let product = Transaction::run(&self.unit_of_work, async {
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

        ReadThrough::invalidate(&self.cache, Invalidation::PRODUCT_WRITE).await?;

        Ok(product)
    }

    /// Remove um produto — soft-delete.
    ///
    /// Confere a existência antes de apagar: sem isso, remover duas vezes
    /// responderia sucesso na segunda, e o cliente não teria como saber que o id
    /// que ele mandou nunca existiu.
    async fn delete(&self, command: DeleteProductCommand) -> Result<(), AppError> {
        self.delete_permission.authorize(&command.context)?;

        Transaction::run(&self.unit_of_work, async {
            self.products
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("produto", &command.id))?;

            self.products.delete(&command.id).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::PRODUCT_WRITE).await?;

        Ok(())
    }

    async fn get(&self, query: GetProductQuery) -> Result<ProductViewItem, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = CacheKey::of(CacheKey::PRODUCT, "get", &[&query.id]);

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.get_product(&query.id)?;

            Transaction::run(&self.unit_of_work, async {
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

        let key = CacheKey::of(
            CacheKey::PRODUCT,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.search.as_deref().unwrap_or_default(),
            ],
        );

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.list_products(ListParams {
                cursor: query.cursor.clone(),
                limit: query.limit,
                search: query.search.clone(),
            });

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
