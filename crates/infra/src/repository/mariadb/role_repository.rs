//! Persistência de papéis sobre `MariaDB`.

use anyhow::Context;
use chrono::Utc;
use mysql_async::params;
use mysql_async::prelude::Queryable as _;
use portmaster_domain::domain::Role;

use crate::entity::codec::Codec;
use crate::entity::role_entity::RoleEntity;
use crate::repository::RoleRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;
use crate::search_key::SearchKey;

/// Monta o repositório de papéis.
///
/// Não guarda estado: a transação vem do escopo da tarefa, não de um campo — o
/// que permite ao provider reconstruí-lo a cada chamada por custo praticamente
/// zero.
pub(super) fn role_repository<T>(
    transactions: T,
) -> impl RoleRepository + Sync + Clone + use<T> + 'static
where
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    RoleMariadbRepository { transactions }
}

/// O repositório de papéis, sobre o `MariaDB`.
#[derive(Clone)]
struct RoleMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T: MySqlTransaction + Send + Sync> RoleRepository for RoleMariadbRepository<T> {
    /// Busca por id, já filtrando o soft-delete.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<RoleEntity> = transaction
            .exec_first(
                "SELECT id, name, permissions, created_at, updated_at, deleted_at \
                 FROM `roles` WHERE id = :id AND deleted_at IS NULL",
                params! { "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao buscar o papel {id}"))?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn Role>))
    }

    /// Os papéis de um usuário, na ordem em que o schema consegue reportá-la.
    ///
    /// A ordem sai pelo id do papel, que é um Snowflake — ordenável por tempo de
    /// criação. É o mais próximo de "ordem de concessão" que o schema guarda, já
    /// que a tabela de ligação não tem coluna de sequência.
    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>> {
        let raw_user = Codec::decode_id(user_id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entities: Vec<RoleEntity> = transaction
            .exec(
                "SELECT r.id, r.name, r.permissions, r.created_at, r.updated_at, r.deleted_at \
                 FROM `roles` r \
                 INNER JOIN `user_roles` ur ON ur.role_id = r.id \
                 WHERE ur.user_id = :user_id AND r.deleted_at IS NULL \
                 ORDER BY r.id",
                params! { "user_id" => raw_user },
            )
            .await
            .with_context(|| format!("falha ao buscar os papéis do usuário {user_id}"))?;

        Ok(entities
            .into_iter()
            .map(|entity| Box::new(entity) as Box<dyn Role>)
            .collect())
    }

    /// Grava a linha nova, com os dois instantes que o modelo já carimbou.
    async fn insert(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `roles` (id, name, permissions, search_name, created_at, updated_at) \
                 VALUES (:id, :name, :permissions, :search_name, :created_at, :updated_at)",
                params! {
                    "id" => entity.raw_id(),
                    "name" => entity.name(),
                    "permissions" => entity.permissions_json()?,
                    "search_name" => SearchKey::of(entity.name()),
                    "created_at" => entity.created_at().timestamp_millis(),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                },
            )
            .await
            .with_context(|| format!("falha ao gravar o papel {}", role.id()))?;

        Ok(())
    }

    /// Atualiza a linha existente.
    async fn update(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `roles` SET name = :name, permissions = :permissions, \
                 search_name = :search_name, updated_at = :updated_at \
                 WHERE id = :id AND deleted_at IS NULL",
                params! {
                    "name" => entity.name(),
                    "permissions" => entity.permissions_json()?,
                    "search_name" => SearchKey::of(entity.name()),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                    "id" => entity.raw_id(),
                },
            )
            .await
            .with_context(|| format!("falha ao atualizar o papel {}", role.id()))?;

        Ok(())
    }

    /// Marca como removido em vez de apagar — o histórico continua auditável.
    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `roles` SET deleted_at = :now, updated_at = :now \
                 WHERE id = :id AND deleted_at IS NULL",
                params! { "now" => Utc::now().timestamp_millis(), "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao remover o papel {id}"))?;

        Ok(())
    }
}
