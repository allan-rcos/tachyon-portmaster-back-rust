//! Persistência de papéis sobre `MariaDB`.

use anyhow::Context;
use portmaster_domain::domain::Role;

use crate::entity::codec::Codec;
use crate::entity::role_entity::RoleEntity;
use crate::repository::RoleRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;
use crate::search_key::SearchKey;

/// Busca por id, já filtrando o soft-delete.
const FIND_BY_ID: &str = "SELECT id, name, permissions, created_at, updated_at, deleted_at \
     FROM `roles` WHERE id = ? AND deleted_at IS NULL";

/// A ordem sai pelo id do papel, que é um Snowflake — ordenável por tempo de
/// criação. É o mais próximo de "ordem de concessão" que o schema guarda, já que
/// a tabela de ligação não tem coluna de sequência.
const FIND_BY_USER: &str =
    "SELECT r.id, r.name, r.permissions, r.created_at, r.updated_at, r.deleted_at \
     FROM `roles` r \
     INNER JOIN `user_roles` ur ON ur.role_id = r.id \
     WHERE ur.user_id = ? AND r.deleted_at IS NULL \
     ORDER BY r.id";

/// Grava a linha nova.
const INSERT: &str = "INSERT INTO `roles` (id, name, permissions, search_name) VALUES (?, ?, ?, ?)";

/// Atualiza a linha existente.
const UPDATE: &str = "UPDATE `roles` SET name = ?, permissions = ?, search_name = ? \
                      WHERE id = ? AND deleted_at IS NULL";

/// Marca como removida em vez de apagar — o histórico continua auditável.
const SOFT_DELETE: &str =
    "UPDATE `roles` SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL";

/// O repositório de papéis.
#[derive(Clone)]
pub struct RoleMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T> RoleMariadbRepository<T> {
    /// Monta o repositório.
    pub(crate) const fn new(transactions: T) -> Self {
        Self { transactions }
    }
}

impl<T: MySqlTransaction + Send + Sync> RoleRepository for RoleMariadbRepository<T> {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<RoleEntity> = sqlx::query_as(FIND_BY_ID)
            .bind(raw_id)
            .fetch_optional(&mut **transaction)
            .await
            .with_context(|| format!("falha ao buscar o papel {id}"))?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn Role>))
    }

    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>> {
        let raw_user = Codec::decode_id(user_id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entities: Vec<RoleEntity> = sqlx::query_as(FIND_BY_USER)
            .bind(raw_user)
            .fetch_all(&mut **transaction)
            .await
            .with_context(|| format!("falha ao buscar os papéis do usuário {user_id}"))?;

        Ok(entities
            .into_iter()
            .map(|entity| Box::new(entity) as Box<dyn Role>)
            .collect())
    }

    async fn insert(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.name())
            .bind(entity.permissions_json()?)
            .bind(SearchKey::of(entity.name()))
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao gravar o papel {}", role.id()))?;

        Ok(())
    }

    async fn update(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(UPDATE)
            .bind(entity.name())
            .bind(entity.permissions_json()?)
            .bind(SearchKey::of(entity.name()))
            .bind(entity.raw_id())
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao atualizar o papel {}", role.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao remover o papel {id}"))?;

        Ok(())
    }
}
