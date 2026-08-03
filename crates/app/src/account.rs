//! O que um usuário faz com a própria conta.
//!
//! Nenhum destes exige permissão. Não é esquecimento: o alvo é sempre quem
//! está agindo — o id vem do [`UserContext`], nunca do corpo do request —, e
//! ter sessão válida já é a autorização. Exigir uma permissão aqui significaria
//! que um administrador poderia revogar de alguém o direito de trocar a própria
//! senha, o que trancaria a conta em vez de protegê-la.

use portmaster_domain::auth::AuthTM;
use portmaster_domain::user::{User, UserTM};
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::AccountView;
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::UserRepository;

use crate::cache::{self, prefix};
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Alterar o próprio nome e e-mail.
#[derive(Debug, Clone)]
pub struct UpdateAccountCommand {
    /// Quem está agindo — e sobre quem se age.
    pub context: UserContext,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
}

/// Trocar a própria senha.
#[derive(Debug, Clone)]
pub struct ChangePasswordCommand {
    /// Quem está agindo — e sobre quem se age.
    pub context: UserContext,
    /// A senha atual, para provar posse da conta.
    pub current_password: String,
    /// A senha nova.
    pub new_password: String,
}

/// Ler a própria conta.
#[derive(Debug, Clone)]
pub struct GetAccountQuery {
    /// Quem está consultando — e de quem é a conta.
    pub context: UserContext,
}

/// O que a apresentação pode pedir sobre a conta do próprio usuário.
#[trait_variant::make(Send)]
pub trait AccountUseCase {
    /// Lê a conta de quem está na sessão.
    async fn get(&self, query: GetAccountQuery) -> Result<AccountView, AppError>;

    /// Altera nome e e-mail, e devolve a conta atualizada.
    async fn update(&self, command: UpdateAccountCommand) -> Result<Box<dyn User>, AppError>;

    /// Troca a senha, exigindo a atual.
    async fn change_password(&self, command: ChangePasswordCommand) -> Result<(), AppError>;
}

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
    pub(crate) fn new(
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
        let key = cache::key(prefix::ACCOUNT, "get", &[&query.context.id]);

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.get_account(&query.context.id)?;

            transaction(&self.unit_of_work, async {
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
        let user = transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await?;

        Ok(user)
    }

    async fn change_password(&self, command: ChangePasswordCommand) -> Result<(), AppError> {
        transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await
    }
}
