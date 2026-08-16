//! Persistência de contêineres sobre `MariaDB`.

use anyhow::Context;
use portmaster_domain::domain::Container;

use crate::entity::codec::Codec;
use crate::entity::container_entity::ContainerEntity;
use crate::repository::ContainerRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;
use crate::search_key::SearchKey;

/// Busca por id, já filtrando o soft-delete.
const FIND_BY_ID: &str =
    "SELECT id, code, current_weight, max_capacity, status, created_at, updated_at, deleted_at \
     FROM `containers` WHERE id = ? AND deleted_at IS NULL";

/// Grava a linha nova.
const INSERT: &str = "INSERT INTO `containers` \
                      (id, code, current_weight, max_capacity, status, search_code) \
                      VALUES (?, ?, ?, ?, ?, ?)";

/// Peso e status vão no mesmo comando porque mudam juntos: um movimento de carga
/// altera os dois, e gravá-los separados deixaria uma janela com peso novo e
/// status velho.
const UPDATE: &str = "UPDATE `containers` \
                      SET code = ?, current_weight = ?, max_capacity = ?, status = ?, search_code = ? \
                      WHERE id = ? AND deleted_at IS NULL";

/// Marca como removida em vez de apagar — o histórico continua auditável.
const SOFT_DELETE: &str =
    "UPDATE `containers` SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL";

/// O repositório de contêineres.
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<ContainerEntity> = sqlx::query_as(FIND_BY_ID)
            .bind(raw_id)
            .fetch_optional(&mut **transaction)
            .await
            .with_context(|| format!("falha ao buscar o contêiner {id}"))?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn Container>))
    }

    async fn insert(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.code())
            .bind(entity.current_weight())
            .bind(entity.max_capacity())
            .bind(entity.status().as_i32())
            .bind(SearchKey::of(entity.code()))
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao gravar o contêiner {}", container.id()))?;

        Ok(())
    }

    async fn update(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(UPDATE)
            .bind(entity.code())
            .bind(entity.current_weight())
            .bind(entity.max_capacity())
            .bind(entity.status().as_i32())
            .bind(SearchKey::of(entity.code()))
            .bind(entity.raw_id())
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao atualizar o contêiner {}", container.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao remover o contêiner {id}"))?;

        Ok(())
    }
}
