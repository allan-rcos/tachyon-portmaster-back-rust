//! A orquestração da conta do próprio usuário.

use portmaster_domain::domain::User;
use portmaster_domain::table_modules::AuthTM;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::query::views::AccountView;
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::{UserRepository, ViewCacheRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::account::ChangePasswordCommand;
use crate::commands::account::UpdateAccountCommand;
use crate::error::AccountError;
use crate::queries::account::GetAccountQuery;
use crate::services::AccountService;

/// O prefixo de toda leitura deste serviço.
const CACHE_GROUP: &str = "account";

/// A chave do perfil de quem está na sessão.
/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct AccountServiceImpl<R, T, A, Q, C> {
    /// Persistência de usuários.
    users: R,
    /// As regras de usuário — quem constrói e valida.
    user_tm: T,
    /// As regras de credencial.
    auth_tm: A,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
}

impl<R, T, A, Q, C> AccountServiceImpl<R, T, A, Q, C> {
    /// Monta o caso de uso.
    pub(crate) const fn new(users: R, user_tm: T, auth_tm: A, queries: Q, views: C) -> Self {
        Self {
            users,
            user_tm,
            auth_tm,
            queries,
            views,
        }
    }
}

impl<R, T, A, Q, C> AccountService for AccountServiceImpl<R, T, A, Q, C>
where
    R: UserRepository + Send + Sync,
    T: UserTM + Send + Sync,
    A: AuthTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    /// O perfil de quem está na sessão.
    ///
    /// Não confere permissão: a conta é do próprio usuário, e a sessão já provou
    /// quem ele é. Responde `InvalidCredentials` se ela não existe mais — a
    /// conta some entre a emissão do token e o pedido quando o usuário é
    /// removido, e o token continua assinado sem descrever ninguém.
    async fn get(&self, query: GetAccountQuery) -> Result<AccountView, AccountError> {
        let dql = dql::get_account(&query.context.id)?;
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let Some(view) = self.queries.run(dql).await? else {
                return Err(AccountError::InvalidCredentials);
            };

            uow.commit().await?;

            Ok(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }

    async fn update(&self, command: UpdateAccountCommand) -> Result<Box<dyn User>, AccountError> {
        let user = MasterScope::run(|uow| async move {
            let Some(existing) = self.users.find_by_id(&command.context.id).await? else {
                return Err(AccountError::InvalidCredentials);
            };

            let updated = self
                .user_tm
                .update(existing.as_ref(), command.name, command.email)?;

            self.users.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(user)
    }

    /// Troca a senha do próprio usuário.
    ///
    /// A senha **atual** é exigida mesmo com a sessão válida: um token roubado
    /// não deve bastar para trocar a senha e expulsar o dono.
    async fn change_password(&self, command: ChangePasswordCommand) -> Result<(), AccountError> {
        MasterScope::run(|uow| async move {
            let Some(existing) = self.users.find_by_id(&command.context.id).await? else {
                return Err(AccountError::InvalidCredentials);
            };

            self.auth_tm
                .login(existing.as_ref(), &command.current_password)?;

            let updated = self
                .user_tm
                .change_password(existing.as_ref(), command.new_password)?;

            self.users.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }
}
