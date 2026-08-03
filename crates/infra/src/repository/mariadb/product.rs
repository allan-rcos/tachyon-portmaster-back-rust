//! Persistência de produtos sobre MariaDB.

use anyhow::Context;
use portmaster_domain::product::Product;

use crate::database::uow::MariadbUnitOfWork;
use crate::entity::decode_id;
use crate::entity::product::{ProductEntity, ProductRow};
use crate::repository::ProductRepository;
use crate::text::search_key;

/// Toda leitura filtra `deleted_at IS NULL` — sem isso, um produto removido
/// reapareceria nas consultas.
const FIND_BY_ID: &str =
    "SELECT id, name, density, risk_class, created_at, updated_at, deleted_at \
                          FROM `products` WHERE id = ? AND deleted_at IS NULL";

const INSERT: &str =
    "INSERT INTO `products` (id, name, density, risk_class, search_name) VALUES (?, ?, ?, ?, ?)";

const UPDATE: &str =
    "UPDATE `products` SET name = ?, density = ?, risk_class = ?, search_name = ? \
                      WHERE id = ? AND deleted_at IS NULL";

/// Soft-delete: a linha permanece e é o filtro das leituras que a esconde. Um
/// produto apagado de verdade quebraria o histórico de manifesto que o
/// referencia.
const SOFT_DELETE: &str =
    "UPDATE `products` SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL";

/// O repositório de produtos.
pub(crate) struct ProductMariadbRepository;

impl ProductMariadbRepository {
    /// Monta o repositório.
    ///
    /// Não guarda estado: a transação vem do escopo da requisição, não de um
    /// campo — o que permite ao provider reconstruí-lo a cada chamada por custo
    /// praticamente zero.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl ProductRepository for ProductMariadbRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>> {
        let raw_id = decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        let row: Option<ProductRow> = sqlx::query_as(FIND_BY_ID)
            .bind(raw_id)
            .fetch_optional(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao buscar o produto {id}"))?;

        row.map(ProductEntity::from_row)
            .transpose()
            .map(|entity| entity.map(|e| Box::new(e) as Box<dyn Product>))
    }

    async fn insert(&self, product: &dyn Product) -> anyhow::Result<()> {
        let entity = ProductEntity::from_domain(product)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.name())
            .bind(entity.density())
            .bind(entity.risk_class().as_i32())
            .bind(search_key(entity.name()))
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao gravar o produto {}", product.id()))?;

        Ok(())
    }

    async fn update(&self, product: &dyn Product) -> anyhow::Result<()> {
        let entity = ProductEntity::from_domain(product)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(UPDATE)
            .bind(entity.name())
            .bind(entity.density())
            .bind(entity.risk_class().as_i32())
            .bind(search_key(entity.name()))
            .bind(entity.raw_id())
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao atualizar o produto {}", product.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao remover o produto {id}"))?;

        Ok(())
    }
}
