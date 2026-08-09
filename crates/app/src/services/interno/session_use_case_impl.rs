//! A orquestração de sessão.

use crate::commands::session::LoginCommand;
use crate::commands::session::SetupCommand;
use crate::context::UserContext;
use crate::error::AppError;
use crate::services::SessionUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::User;
use portmaster_domain::table_modules::AuthTM;
use portmaster_domain::table_modules::RoleTM;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::repository::{PermissionRepository, RoleRepository, UserRepository};

/// O nome do papel que o setup cria.
const ADMINISTRATOR_ROLE: &str = "Administrator";

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct SessionUseCaseImpl<UR, RR, PR, UT, RT, A, U> {
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
    /// Quem abre e fecha a transação.
    unit_of_work: U,
}

impl<UR, RR, PR, UT, RT, A, U> SessionUseCaseImpl<UR, RR, PR, UT, RT, A, U> {
    /// Monta o caso de uso.
    pub(crate) const fn new(
        users: UR,
        roles: RR,
        permissions: PR,
        user_tm: UT,
        role_tm: RT,
        auth_tm: A,
        unit_of_work: U,
    ) -> Self {
        Self {
            users,
            roles,
            permissions,
            user_tm,
            role_tm,
            auth_tm,
            unit_of_work,
        }
    }
}

impl<UR, RR, PR, UT, RT, A, U> SessionUseCase for SessionUseCaseImpl<UR, RR, PR, UT, RT, A, U>
where
    UR: UserRepository + Send + Sync,
    RR: RoleRepository + Send + Sync,
    PR: PermissionRepository + Send + Sync,
    UT: UserTM + Send + Sync,
    RT: RoleTM + Send + Sync,
    A: AuthTM + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// Confere a credencial e devolve o usuário.
    ///
    /// E-mail desconhecido cai no **mesmo** erro da senha errada. Distinguir os
    /// dois transformaria a tela de login num verificador de quem tem conta no
    /// sistema.
    async fn login(&self, command: LoginCommand) -> Result<Box<dyn User>, AppError> {
        Transaction::run(&self.unit_of_work, async {
            let Some(user) = self.users.find_by_email(&command.email).await? else {
                return Err(AppError::InvalidCredentials);
            };

            self.auth_tm.login(user.as_ref(), &command.password)?;

            Ok(user)
        })
        .await
    }

    async fn validate(&self, context: &UserContext) -> Result<Box<dyn User>, AppError> {
        Transaction::run(&self.unit_of_work, async {
            self.users
                .find_by_id(&context.id)
                .await?
                .ok_or(AppError::InvalidCredentials)
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
    async fn setup(&self, command: SetupCommand) -> Result<Box<dyn User>, AppError> {
        Transaction::run(&self.unit_of_work, async {
            if self.users.has_any().await? {
                return Err(AppError::RuleViolation(
                    "This system has already been set up.".into(),
                ));
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

            Ok(user)
        })
        .await
    }
}
