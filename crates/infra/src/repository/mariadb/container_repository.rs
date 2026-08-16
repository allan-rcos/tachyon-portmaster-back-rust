//! Persistência de contêineres sobre `MariaDB`.

use anyhow::Context;
use chrono::Utc;
use mysql_async::params;
use mysql_async::prelude::Queryable as _;
use portmaster_domain::domain::Container;

use crate::entity::codec::Codec;
use crate::entity::container_entity::ContainerEntity;
use crate::repository::ContainerRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;
use crate::search_key::SearchKey;

/// Monta o repositório de contêineres.
///
/// Não guarda estado: a transação vem do escopo da tarefa, não de um campo — o
/// que permite ao provider reconstruí-lo a cada chamada por custo praticamente
/// zero.
pub(super) fn container_repository<T>(
    transactions: T,
) -> impl ContainerRepository + Sync + Clone + use<T> + 'static
where
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    ContainerMariadbRepository { transactions }
}

/// O repositório de contêineres, sobre o `MariaDB`.
#[derive(Clone)]
struct ContainerMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T: MySqlTransaction + Send + Sync> ContainerRepository for ContainerMariadbRepository<T> {
    /// Busca por id, já filtrando o soft-delete.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<ContainerEntity> = transaction
            .exec_first(
                "SELECT id, code, current_weight, max_capacity, status, \
                 created_at, updated_at, deleted_at \
                 FROM `containers` WHERE id = :id AND deleted_at IS NULL",
                params! { "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao buscar o contêiner {id}"))?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn Container>))
    }

    /// Grava a linha nova, com os dois instantes que o modelo já carimbou.
    async fn insert(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `containers` \
                 (id, code, current_weight, max_capacity, status, search_code, \
                  created_at, updated_at) \
                 VALUES (:id, :code, :current_weight, :max_capacity, :status, :search_code, \
                         :created_at, :updated_at)",
                params! {
                    "id" => entity.raw_id(),
                    "code" => entity.code(),
                    "current_weight" => entity.current_weight(),
                    "max_capacity" => entity.max_capacity(),
                    "status" => entity.status().as_i32(),
                    "search_code" => SearchKey::of(entity.code()),
                    "created_at" => entity.created_at().timestamp_millis(),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                },
            )
            .await
            .with_context(|| format!("falha ao gravar o contêiner {}", container.id()))?;

        Ok(())
    }

    /// Peso e status vão no mesmo comando porque mudam juntos.
    ///
    /// Um movimento de carga altera os dois, e gravá-los separados deixaria uma
    /// janela com peso novo e status velho.
    async fn update(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `containers` SET code = :code, current_weight = :current_weight, \
                 max_capacity = :max_capacity, status = :status, search_code = :search_code, \
                 updated_at = :updated_at \
                 WHERE id = :id AND deleted_at IS NULL",
                params! {
                    "code" => entity.code(),
                    "current_weight" => entity.current_weight(),
                    "max_capacity" => entity.max_capacity(),
                    "status" => entity.status().as_i32(),
                    "search_code" => SearchKey::of(entity.code()),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                    "id" => entity.raw_id(),
                },
            )
            .await
            .with_context(|| format!("falha ao atualizar o contêiner {}", container.id()))?;

        Ok(())
    }

    /// Marca como removido em vez de apagar — o histórico continua auditável.
    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `containers` SET deleted_at = :now, updated_at = :now \
                 WHERE id = :id AND deleted_at IS NULL",
                params! { "now" => Utc::now().timestamp_millis(), "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao remover o contêiner {id}"))?;

        Ok(())
    }
}
