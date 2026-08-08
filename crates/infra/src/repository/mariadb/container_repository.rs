//! Persistência de contêineres sobre `MariaDB`.

use anyhow::Context;
use portmaster_domain::models::Container;

use crate::database::interno::mariadb_unit_of_work::MariadbUnitOfWork;
use crate::entity::codec::Codec;
use crate::entity::container_entity::ContainerEntity;
use crate::entity::container_row::ContainerRow;
use crate::repository::ContainerRepository;
use crate::text::search_key::SearchKey;

const FIND_BY_ID: &str =
    "SELECT id, code, current_weight, max_capacity, status, created_at, updated_at, deleted_at \
     FROM `containers` WHERE id = ? AND deleted_at IS NULL";

const INSERT: &str = "INSERT INTO `containers` \
                      (id, code, current_weight, max_capacity, status, search_code) \
                      VALUES (?, ?, ?, ?, ?, ?)";

/// Peso e status vão no mesmo comando porque mudam juntos: um movimento de carga
/// altera os dois, e gravá-los separados deixaria uma janela com peso novo e
/// status velho.
const UPDATE: &str = "UPDATE `containers` \
                      SET code = ?, current_weight = ?, max_capacity = ?, status = ?, search_code = ? \
                      WHERE id = ? AND deleted_at IS NULL";

const SOFT_DELETE: &str =
    "UPDATE `containers` SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL";

/// O repositório de contêineres.
pub struct ContainerMariadbRepository;

impl ContainerMariadbRepository {
    /// Monta o repositório.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl ContainerRepository for ContainerMariadbRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        let row: Option<ContainerRow> = sqlx::query_as(FIND_BY_ID)
            .bind(raw_id)
            .fetch_optional(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao buscar o contêiner {id}"))?;

        row.map(ContainerEntity::from_row)
            .transpose()
            .map(|entity| entity.map(|e| Box::new(e) as Box<dyn Container>))
    }

    async fn insert(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.code())
            .bind(entity.current_weight())
            .bind(entity.max_capacity())
            .bind(entity.status().as_i32())
            .bind(SearchKey::of(entity.code()))
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao gravar o contêiner {}", container.id()))?;

        Ok(())
    }

    async fn update(&self, container: &dyn Container) -> anyhow::Result<()> {
        let entity = ContainerEntity::from_domain(container)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(UPDATE)
            .bind(entity.code())
            .bind(entity.current_weight())
            .bind(entity.max_capacity())
            .bind(entity.status().as_i32())
            .bind(SearchKey::of(entity.code()))
            .bind(entity.raw_id())
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao atualizar o contêiner {}", container.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao remover o contêiner {id}"))?;

        Ok(())
    }
}
