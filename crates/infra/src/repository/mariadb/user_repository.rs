//! Persistência de usuários sobre `MariaDB`.

use anyhow::Context;
use chrono::Utc;
use mysql_async::params;
use mysql_async::prelude::Queryable as _;
use portmaster_domain::domain::User;

use crate::entity::codec::Codec;
use crate::entity::user_entity::UserEntity;
use crate::repository::{RoleRepository, UserRepository};
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// Monta o repositório de usuários sobre o de papéis.
///
/// O de papéis chega injetado porque um usuário não está completo sem eles: os
/// papéis decidem o que ele pode fazer, e devolvê-lo sem papéis faria toda
/// verificação de autorização falhar em silêncio.
pub(super) fn user_repository<R, T>(
    roles: R,
    transactions: T,
) -> impl UserRepository + Sync + Clone + use<R, T> + 'static
where
    R: RoleRepository + Send + Sync + Clone + 'static,
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    UserMariadbRepository {
        roles,
        transactions,
    }
}

/// O repositório de usuários, genérico sobre o de papéis.
#[derive(Clone)]
struct UserMariadbRepository<R, T> {
    /// De onde os papéis do usuário são lidos, na mesma leitura.
    roles: R,
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<R: RoleRepository, T> UserMariadbRepository<R, T> {
    /// Completa a entity buscando os papéis do usuário.
    ///
    /// A busca dos papéis acontece **depois** de soltar o empréstimo da
    /// transação: as duas consultas usam a mesma conexão, e segurá-la durante a
    /// segunda travaria a si mesma.
    async fn hydrate(&self, entity: UserEntity) -> anyhow::Result<UserEntity> {
        let roles = self.roles.find_by_user_id(entity.id()).await?;
        Ok(entity.with_roles(roles))
    }
}

impl<R: RoleRepository + Send + Sync, T: MySqlTransaction + Send + Sync> UserRepository
    for UserMariadbRepository<R, T>
{
    /// `LIMIT 1` e não `COUNT(*)`: a pergunta é "existe algum", e contar todos
    /// para descobrir isso varre a tabela inteira à toa.
    async fn has_any(&self) -> anyhow::Result<bool> {
        let mut transaction = self.transactions.transaction().await?;

        let found: Option<i64> = transaction
            .exec_first("SELECT 1 FROM `users` WHERE deleted_at IS NULL LIMIT 1", ())
            .await
            .context("falha ao verificar se há algum usuário")?;

        Ok(found.is_some())
    }

    /// Busca por id, já filtrando o soft-delete.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn User>>> {
        let raw_id = Codec::decode_id(id)?;

        let entity: Option<UserEntity> = {
            let mut transaction = self.transactions.transaction().await?;
            transaction
                .exec_first(
                    "SELECT id, name, email, password_hash, created_at, updated_at, deleted_at \
                     FROM `users` WHERE id = :id AND deleted_at IS NULL",
                    params! { "id" => raw_id },
                )
                .await
                .with_context(|| format!("falha ao buscar o usuário {id}"))?
        };

        match entity {
            Some(entity) => Ok(Some(Box::new(self.hydrate(entity).await?))),
            None => Ok(None),
        }
    }

    /// O filtro de removidos vale também aqui: um e-mail liberado por remoção
    /// precisa poder ser reusado.
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Box<dyn User>>> {
        let entity: Option<UserEntity> = {
            let mut transaction = self.transactions.transaction().await?;
            transaction
                .exec_first(
                    "SELECT id, name, email, password_hash, created_at, updated_at, deleted_at \
                     FROM `users` WHERE email = :email AND deleted_at IS NULL",
                    params! { "email" => email },
                )
                .await
                .context("falha ao buscar usuário por e-mail")?
        };

        match entity {
            Some(entity) => Ok(Some(Box::new(self.hydrate(entity).await?))),
            None => Ok(None),
        }
    }

    /// Grava a linha nova, com os dois instantes que o modelo já carimbou.
    async fn insert(&self, user: &dyn User) -> anyhow::Result<()> {
        let entity = UserEntity::from_domain(user)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `users` (id, name, email, password_hash, created_at, updated_at) \
                 VALUES (:id, :name, :email, :password_hash, :created_at, :updated_at)",
                params! {
                    "id" => entity.raw_id(),
                    "name" => entity.name(),
                    "email" => entity.email(),
                    "password_hash" => entity.password_hash(),
                    "created_at" => entity.created_at().timestamp_millis(),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                },
            )
            .await
            .with_context(|| format!("falha ao gravar o usuário {}", user.id()))?;

        Ok(())
    }

    /// Atualiza a linha existente.
    async fn update(&self, user: &dyn User) -> anyhow::Result<()> {
        let entity = UserEntity::from_domain(user)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `users` SET name = :name, email = :email, \
                 password_hash = :password_hash, updated_at = :updated_at \
                 WHERE id = :id AND deleted_at IS NULL",
                params! {
                    "name" => entity.name(),
                    "email" => entity.email(),
                    "password_hash" => entity.password_hash(),
                    "updated_at" => entity.updated_at().timestamp_millis(),
                    "id" => entity.raw_id(),
                },
            )
            .await
            .with_context(|| format!("falha ao atualizar o usuário {}", user.id()))?;

        Ok(())
    }

    /// Marca como removido em vez de apagar — o histórico continua auditável.
    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let raw_id = Codec::decode_id(id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "UPDATE `users` SET deleted_at = :now, updated_at = :now \
                 WHERE id = :id AND deleted_at IS NULL",
                params! { "now" => Utc::now().timestamp_millis(), "id" => raw_id },
            )
            .await
            .with_context(|| format!("falha ao remover o usuário {id}"))?;

        Ok(())
    }

    /// Substitui os vínculos de papel do usuário — apaga e regrava.
    ///
    /// O vínculo é entidade fraca, e "mudar" um conjunto de vínculos é
    /// removê-los e recriá-los. Calcular o diferencial daria o mesmo resultado
    /// por mais trabalho, e os dois rodam na mesma transação.
    async fn sync_roles(&self, user_id: &str, role_ids: &[String]) -> anyhow::Result<()> {
        let raw_user = Codec::decode_id(user_id)?;
        let raw_roles: Vec<i64> = role_ids
            .iter()
            .map(|id| Codec::decode_id(id))
            .collect::<anyhow::Result<_>>()?;

        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "DELETE FROM `user_roles` WHERE user_id = :user_id",
                params! { "user_id" => raw_user },
            )
            .await
            .with_context(|| format!("falha ao limpar os papéis do usuário {user_id}"))?;

        for role_id in raw_roles {
            transaction
                .exec_drop(
                    "INSERT INTO `user_roles` (user_id, role_id) VALUES (:user_id, :role_id)",
                    params! { "user_id" => raw_user, "role_id" => role_id },
                )
                .await
                .with_context(|| format!("falha ao vincular papel ao usuário {user_id}"))?;
        }

        Ok(())
    }
}
