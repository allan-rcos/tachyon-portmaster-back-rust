//! A orquestração da conta do próprio usuário.

use crate::cache::cache_key::CacheKey;
use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::account::ChangePasswordCommand;
use crate::commands::account::UpdateAccountCommand;
use crate::error::AppError;
use crate::queries::account::GetAccountQuery;
use crate::services::AccountUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::User;
use portmaster_domain::table_modules::AuthTM;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::AccountView;
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::UserRepository;

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct AccountUseCaseImpl<R, T, A, Q, F, C, U> {
    users: R,
    user_tm: T,
    auth_tm: A,
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
}

impl<R, T, A, Q, F, C, U> AccountUseCaseImpl<R, T, A, Q, F, C, U> {
    /// Monta o caso de uso.
    pub(crate) const fn new(
        users: R,
        user_tm: T,
        auth_tm: A,
        queries: Q,
        dqls: F,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            users,
            user_tm,
            auth_tm,
            queries,
            dqls,
            cache,
            unit_of_work,
        }
    }
}

impl<R, T, A, Q, F, C, U> AccountUseCase for AccountUseCaseImpl<R, T, A, Q, F, C, U>
where
    R: UserRepository + Send + Sync,
    T: UserTM + Send + Sync,
    A: AuthTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn get(&self, query: GetAccountQuery) -> Result<AccountView, AppError> {
        let key = CacheKey::of(CacheKey::ACCOUNT, "get", &[&query.context.id]);

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.get_account(&query.context.id)?;

            Transaction::run(&self.unit_of_work, async {
                // A conta some entre a emissão do token e o pedido quando o
                // usuário foi removido — o token continua assinado e válido, mas
                // já não descreve ninguém.
                self.queries
                    .run(dql)
                    .await?
                    .ok_or(AppError::Unauthenticated)
            })
            .await
        })
        .await
    }

    async fn update(&self, command: UpdateAccountCommand) -> Result<Box<dyn User>, AppError> {
        let user = Transaction::run(&self.unit_of_work, async {
            let existing = self
                .users
                .find_by_id(&command.context.id)
                .await?
                .ok_or(AppError::Unauthenticated)?;

            let updated = self
                .user_tm
                .update(existing.as_ref(), command.name, command.email)?;

            self.users.update(updated.as_ref()).await?;

            Ok(updated)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await?;

        Ok(user)
    }

    async fn change_password(&self, command: ChangePasswordCommand) -> Result<(), AppError> {
        Transaction::run(&self.unit_of_work, async {
            let existing = self
                .users
                .find_by_id(&command.context.id)
                .await?
                .ok_or(AppError::Unauthenticated)?;

            // A senha atual é exigida mesmo com a sessão válida: um token
            // roubado não deve bastar para trocar a senha e expulsar o dono.
            self.auth_tm
                .login(existing.as_ref(), &command.current_password)?;

            let updated = self
                .user_tm
                .change_password(existing.as_ref(), command.new_password)?;

            self.users.update(updated.as_ref()).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await
    }
}
