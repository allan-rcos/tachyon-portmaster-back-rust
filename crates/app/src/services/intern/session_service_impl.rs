//! A orquestração de sessão.

use portmaster_domain::domain::User;
use portmaster_domain::table_modules::AuthTM;
use portmaster_domain::table_modules::RoleTM;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::repository::{PermissionRepository, RoleRepository, UserRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::session::LoginCommand;
use crate::commands::session::SetupCommand;
use crate::context::UserContext;
use crate::error::SessionError;
use crate::services::SessionService;

/// O nome do papel que o setup cria.
const ADMINISTRATOR_ROLE: &str = "Administrator";

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct SessionServiceImpl<UR, RR, PR, UT, RT, A> {
    /// Persistência de usuários.
    users: UR,
    /// Persistência de papéis.
    roles: RR,
    /// O catálogo de permissões, em memória.
    permissions: PR,
    /// As regras de usuário — quem constrói e valida.
    user_tm: UT,
    /// As regras de papel.
    role_tm: RT,
    /// As regras de credencial.
    auth_tm: A,
}

impl<UR, RR, PR, UT, RT, A> SessionServiceImpl<UR, RR, PR, UT, RT, A> {
    /// Monta o caso de uso.
    pub(crate) const fn new(
        users: UR,
        roles: RR,
        permissions: PR,
        user_tm: UT,
        role_tm: RT,
        auth_tm: A,
    ) -> Self {
        Self {
            users,
            roles,
            permissions,
            user_tm,
            role_tm,
            auth_tm,
        }
    }
}

impl<UR, RR, PR, UT, RT, A> SessionService for SessionServiceImpl<UR, RR, PR, UT, RT, A>
where
    UR: UserRepository + Send + Sync,
    RR: RoleRepository + Send + Sync,
    PR: PermissionRepository + Send + Sync,
    UT: UserTM + Send + Sync,
    RT: RoleTM + Send + Sync,
    A: AuthTM + Send + Sync,
{
    /// Confere a credencial e devolve o usuário.
    ///
    /// E-mail desconhecido cai no **mesmo** erro da senha errada. Distinguir os
    /// dois transformaria a tela de login num verificador de quem tem conta no
    /// sistema.
    async fn login(&self, command: LoginCommand) -> Result<Box<dyn User>, SessionError> {
        MasterScope::run(|uow| async move {
            let Some(user) = self.users.find_by_email(&command.email).await? else {
                return Err(SessionError::InvalidCredentials);
            };

            self.auth_tm.login(user.as_ref(), &command.password)?;

            uow.commit().await?;

            Ok(user)
        })
        .await
    }

    async fn validate(&self, context: &UserContext) -> Result<Box<dyn User>, SessionError> {
        MasterScope::run(|uow| async move {
            let Some(user) = self.users.find_by_id(&context.id).await? else {
                return Err(SessionError::InvalidCredentials);
            };

            uow.commit().await?;

            Ok(user)
        })
        .await
    }

    /// Cria o primeiro usuário, e fecha o endpoint para sempre.
    ///
    /// Um sistema com **qualquer** usuário já foi montado; sem essa checagem, o
    /// endpoint seria uma porta aberta para criar um administrador a qualquer
    /// momento.
    ///
    /// As permissões concedidas saem do registro preenchido no boot, e não de
    /// uma lista literal aqui: uma permissão nova é concedida ao administrador
    /// sem ninguém lembrar de voltar a este arquivo.
    async fn setup(&self, command: SetupCommand) -> Result<Box<dyn User>, SessionError> {
        MasterScope::run(|uow| async move {
            if self.users.has_any().await? {
                return Err(SessionError::AlreadySetUp);
            }

            let granted = self.permissions.all().await?;

            let role = self
                .role_tm
                .create(ADMINISTRATOR_ROLE.to_owned(), granted)?;

            self.roles.insert(role.as_ref()).await?;

            let role_id = role.id().to_owned();
            let user =
                self.user_tm
                    .create(command.name, command.email, command.password, vec![role])?;

            self.users.insert(user.as_ref()).await?;

            self.users.sync_roles(user.id(), &[role_id]).await?;

            uow.commit().await?;

            Ok(user)
        })
        .await
    }
}

#[cfg(test)]
#[path = "tests/session_service_impl_test.rs"]
mod tests;
