//! Persistência de produtos sobre `MariaDB`.

use anyhow::Context;
use chrono::Utc;
use mysql_async::params;
use mysql_async::prelude::Queryable as _;
use portmaster_domain::domain::Product;

use crate::entity::codec::Codec;
use crate::entity::product_entity::ProductEntity;
use crate::repository::ProductRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;
use crate::search_key::SearchKey;

/// Monta o repositório de produtos.
///
/// Não guarda estado: a transação vem do escopo da tarefa, não de um campo — o
/// que permite ao provider reconstruí-lo a cada chamada por custo praticamente
/// zero.
pub(super) fn product_repository<T>(
    transactions: T,
) -> impl ProductRepository + Sync + Clone + use<T> + 'static
where
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    ProductMariadbRepository { transactions }
}

/// O repositório de produtos, sobre o `MariaDB`.
#[derive(Clone)]
struct ProductMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T: MySqlTransaction + Send + Sync> ProductRepository for ProductMariadbRepository<T> {
    /// Toda leitura filtra `deleted_at IS NULL` — sem isso, um produto removido
    /// reapareceria nas consultas.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<ProductEntity> = transaction
            .exec_first(
                "SELECT id, name, density, risk_class, created_at, updated_at, deleted_at \
                 FROM `products` WHERE id = :id AND deleted_at IS NULL",
                params! { "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao buscar o produto {id}"))?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn Product>))
    }

    /// Grava a linha nova, com os dois instantes que o modelo já carimbou.
    async fn insert(&self, product: &dyn Product) -> anyhow::Result<()> {
        let entity = ProductEntity::from_domain(product)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `products` \
                 (id, name, density, risk_class, search_name, created_at, updated_at) \
                 VALUES (:id, :name, :density, :risk_class, :search_name, :created_at, :updated_at)",
                params! {
                    "id" => entity.raw_id(),
                    "name" => entity.name(),
                    "density" => entity.density(),
                    "risk_class" => entity.risk_class().as_i32(),
                    "search_name" => SearchKey::of(entity.name()),
                    "created_at" => entity.created_at().timestamp_millis(),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                },
            )
            .await
            .with_context(|| format!("falha ao gravar o produto {}", product.id()))?;

        Ok(())
    }

    /// Atualiza a linha existente.
    ///
    /// `updated_at` vem do modelo, que o refaz a cada mutação — não daqui, e não
    /// do relógio do servidor.
    async fn update(&self, product: &dyn Product) -> anyhow::Result<()> {
        let entity = ProductEntity::from_domain(product)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `products` SET name = :name, density = :density, \
                 risk_class = :risk_class, search_name = :search_name, updated_at = :updated_at \
                 WHERE id = :id AND deleted_at IS NULL",
                params! {
                    "name" => entity.name(),
                    "density" => entity.density(),
                    "risk_class" => entity.risk_class().as_i32(),
                    "search_name" => SearchKey::of(entity.name()),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                    "id" => entity.raw_id(),
                },
            )
            .await
            .with_context(|| format!("falha ao atualizar o produto {}", product.id()))?;

        Ok(())
    }

    /// Soft-delete: a linha permanece e é o filtro das leituras que a esconde.
    ///
    /// Um produto apagado de verdade quebraria o histórico de manifesto que o
    /// referencia. O instante é o desta máquina, como todos os outros — a
    /// remoção não passa por um modelo de domínio que o carimbe antes.
    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `products` SET deleted_at = :now, updated_at = :now \
                 WHERE id = :id AND deleted_at IS NULL",
                params! { "now" => Utc::now().timestamp_millis(), "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao remover o produto {id}"))?;

        Ok(())
    }
}
