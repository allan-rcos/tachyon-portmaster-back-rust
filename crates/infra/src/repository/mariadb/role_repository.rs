//! Persistência de papéis sobre `MariaDB`.

use anyhow::Context;
use portmaster_domain::models::Role;

use crate::database::interno::mariadb_unit_of_work::MariadbUnitOfWork;
use crate::entity::codec::Codec;
use crate::entity::role_entity::RoleEntity;
use crate::entity::role_row::RoleRow;
use crate::repository::RoleRepository;
use crate::text::search_key::SearchKey;

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
pub struct RoleMariadbRepository;

impl RoleMariadbRepository {
    /// Monta o repositório.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl RoleRepository for RoleMariadbRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        let row: Option<RoleRow> = sqlx::query_as(FIND_BY_ID)
            .bind(raw_id)
            .fetch_optional(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao buscar o papel {id}"))?;

        row.map(RoleEntity::from_row)
            .transpose()
            .map(|entity| entity.map(|e| Box::new(e) as Box<dyn Role>))
    }

    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>> {
        let raw_user = Codec::decode_id(user_id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        let rows: Vec<RoleRow> = sqlx::query_as(FIND_BY_USER)
            .bind(raw_user)
            .fetch_all(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao buscar os papéis do usuário {user_id}"))?;

        rows.into_iter()
            .map(|row| RoleEntity::from_row(row).map(|e| Box::new(e) as Box<dyn Role>))
            .collect()
    }

    async fn insert(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.name())
            .bind(entity.permissions_json()?)
            .bind(SearchKey::of(entity.name()))
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao gravar o papel {}", role.id()))?;

        Ok(())
    }

    async fn update(&self, role: &dyn Role) -> anyhow::Result<()> {
        let entity = RoleEntity::from_domain(role)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(UPDATE)
            .bind(entity.name())
            .bind(entity.permissions_json()?)
            .bind(SearchKey::of(entity.name()))
            .bind(entity.raw_id())
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao atualizar o papel {}", role.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao remover o papel {id}"))?;

        Ok(())
    }
}
