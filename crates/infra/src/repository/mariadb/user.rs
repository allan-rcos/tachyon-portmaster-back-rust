//! Persistência de usuários sobre MariaDB.

use anyhow::Context;
use portmaster_domain::user::User;

use crate::database::uow::MariadbUnitOfWork;
use crate::entity::user::{UserEntity, UserRow};
use crate::entity::{decode_id, encode_id};
use crate::repository::{RoleRepository, UserRepository};

/// `LIMIT 1` e não `COUNT(*)`: a pergunta é "existe algum", e contar todos para
/// descobrir isso varre a tabela inteira à toa.
const HAS_ANY: &str = "SELECT 1 FROM `users` WHERE deleted_at IS NULL LIMIT 1";

const FIND_BY_ID: &str =
    "SELECT id, name, email, password_hash, created_at, updated_at, deleted_at \
     FROM `users` WHERE id = ? AND deleted_at IS NULL";

/// O filtro de removidos vale também aqui: um e-mail liberado por remoção
/// precisa poder ser reusado.
const FIND_BY_EMAIL: &str =
    "SELECT id, name, email, password_hash, created_at, updated_at, deleted_at \
     FROM `users` WHERE email = ? AND deleted_at IS NULL";

const INSERT: &str = "INSERT INTO `users` (id, name, email, password_hash) VALUES (?, ?, ?, ?)";

const UPDATE: &str = "UPDATE `users` SET name = ?, email = ?, password_hash = ? \
                      WHERE id = ? AND deleted_at IS NULL";

const SOFT_DELETE: &str =
    "UPDATE `users` SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL";

const CLEAR_ROLES: &str = "DELETE FROM `user_roles` WHERE user_id = ?";

const LINK_ROLE: &str = "INSERT INTO `user_roles` (user_id, role_id) VALUES (?, ?)";

/// O repositório de usuários.
///
/// Genérico sobre o repositório de papéis porque um usuário não está completo
/// sem eles: os papéis decidem o que ele pode fazer, e devolvê-lo sem papéis
/// faria toda verificação de autorização falhar em silêncio.
pub(crate) struct UserMariadbRepository<R> {
    roles: R,
}

impl<R: RoleRepository> UserMariadbRepository<R> {
    /// Monta o repositório sobre o de papéis.
    pub(crate) fn new(roles: R) -> Self {
        Self { roles }
    }

    /// Completa a entity buscando os papéis do usuário.
    ///
    /// A busca dos papéis acontece **depois** de soltar o empréstimo da
    /// transação: as duas consultas usam a mesma conexão, e segurá-la durante a
    /// segunda travaria a si mesma.
    async fn hydrate(&self, row: UserRow) -> anyhow::Result<UserEntity> {
        let id = encode_id(row.id);
        let roles = self.roles.find_by_user_id(&id).await?;
        Ok(UserEntity::from_row(row, roles))
    }
}

impl<R: RoleRepository + Send + Sync> UserRepository for UserMariadbRepository<R> {
    async fn has_any(&self) -> anyhow::Result<bool> {
        let mut transaction = MariadbUnitOfWork::current().await?;

        let found: Option<(i64,)> = sqlx::query_as(HAS_ANY)
            .fetch_optional(&mut **transaction.as_mut())
            .await
            .context("falha ao verificar se há algum usuário")?;

        Ok(found.is_some())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn User>>> {
        let raw_id = decode_id(id)?;

        let row: Option<UserRow> = {
            let mut transaction = MariadbUnitOfWork::current().await?;
            sqlx::query_as(FIND_BY_ID)
                .bind(raw_id)
                .fetch_optional(&mut **transaction.as_mut())
                .await
                .with_context(|| format!("falha ao buscar o usuário {id}"))?
        };

        match row {
            Some(row) => Ok(Some(Box::new(self.hydrate(row).await?))),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Box<dyn User>>> {
        let row: Option<UserRow> = {
            let mut transaction = MariadbUnitOfWork::current().await?;
            sqlx::query_as(FIND_BY_EMAIL)
                .bind(email)
                .fetch_optional(&mut **transaction.as_mut())
                .await
                .context("falha ao buscar usuário por e-mail")?
        };

        match row {
            Some(row) => Ok(Some(Box::new(self.hydrate(row).await?))),
            None => Ok(None),
        }
    }

    async fn insert(&self, user: &dyn User) -> anyhow::Result<()> {
        let entity = UserEntity::from_domain(user)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(INSERT)
            .bind(entity.raw_id())
            .bind(entity.name())
            .bind(entity.email())
            .bind(entity.password_hash())
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao gravar o usuário {}", user.id()))?;

        Ok(())
    }

    async fn update(&self, user: &dyn User) -> anyhow::Result<()> {
        let entity = UserEntity::from_domain(user)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(UPDATE)
            .bind(entity.name())
            .bind(entity.email())
            .bind(entity.password_hash())
            .bind(entity.raw_id())
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao atualizar o usuário {}", user.id()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = decode_id(id)?;
        let mut transaction = MariadbUnitOfWork::current().await?;

        sqlx::query(SOFT_DELETE)
            .bind(raw_id)
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao remover o usuário {id}"))?;

        Ok(())
    }

    async fn sync_roles(&self, user_id: &str, role_ids: &[String]) -> anyhow::Result<()> {
        let raw_user = decode_id(user_id)?;
        let raw_roles: Vec<i64> = role_ids
            .iter()
            .map(|id| decode_id(id))
            .collect::<anyhow::Result<_>>()?;

        let mut transaction = MariadbUnitOfWork::current().await?;

        // Apaga e regrava: o vínculo é entidade fraca, e "mudar" um conjunto de
        // vínculos é removê-los e recriá-los. Calcular o diferencial daria o
        // mesmo resultado por mais trabalho, e os dois rodam na mesma transação.
        sqlx::query(CLEAR_ROLES)
            .bind(raw_user)
            .execute(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao limpar os papéis do usuário {user_id}"))?;

        for role_id in raw_roles {
            sqlx::query(LINK_ROLE)
                .bind(raw_user)
                .bind(role_id)
                .execute(&mut **transaction.as_mut())
                .await
                .with_context(|| format!("falha ao vincular papel ao usuário {user_id}"))?;
        }

        Ok(())
    }
}
