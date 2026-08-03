//! Entrar no sistema, e montá-lo pela primeira vez.
//!
//! Nenhum destes exige permissão, e cada um por um motivo diferente: o login
//! acontece **antes** de haver contexto; a validação de sessão só reconfere o
//! que o token já afirma; e o setup roda quando ainda não existe usuário nenhum
//! a quem pedir autorização — é ele quem cria o primeiro.
//!
//! ## O que esta camada não sabe
//!
//! Que existe JWT. O login devolve o usuário com os papéis dele, e é o
//! `api-http` que decide empacotar aquilo num token. Se um dia a sessão virar
//! cookie de servidor, nada aqui muda.

use portmaster_domain::auth::AuthTM;
use portmaster_domain::role::RoleTM;
use portmaster_domain::user::{User, UserTM};
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::repository::{PermissionRepository, RoleRepository, UserRepository};

use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// O nome do papel que o setup cria.
const ADMINISTRATOR_ROLE: &str = "Administrator";

/// Autenticar-se.
///
/// Não carrega contexto: é o comando que **produz** um.
#[derive(Debug, Clone)]
pub struct LoginCommand {
    /// E-mail cadastrado.
    pub email: String,
    /// Senha em claro.
    pub password: String,
}

/// Montar o sistema com o primeiro usuário.
#[derive(Debug, Clone)]
pub struct SetupCommand {
    /// Nome do administrador.
    pub name: String,
    /// E-mail do administrador.
    pub email: String,
    /// Senha do administrador.
    pub password: String,
}

/// O que a apresentação pode pedir sobre sessão.
#[trait_variant::make(Send)]
pub trait SessionUseCase {
    /// Confere as credenciais e devolve o usuário com os papéis dele.
    async fn login(&self, command: LoginCommand) -> Result<Box<dyn User>, AppError>;

    /// Reconfere que a sessão ainda descreve alguém.
    ///
    /// O token é auto-contido e válido até expirar, então ele continua sendo
    /// aceito depois de o usuário ser removido ou ter os papéis trocados. Isto é
    /// o que confronta a afirmação do token com o banco, para quem quiser pagar
    /// a consulta.
    async fn validate(&self, context: &UserContext) -> Result<Box<dyn User>, AppError>;

    /// Cria o primeiro usuário e o papel que concede tudo.
    async fn setup(&self, command: SetupCommand) -> Result<Box<dyn User>, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct SessionUseCaseImpl<UR, RR, PR, UT, RT, A, U> {
    users: UR,
    roles: RR,
    permissions: PR,
    user_tm: UT,
    role_tm: RT,
    auth_tm: A,
    unit_of_work: U,
}

impl<UR, RR, PR, UT, RT, A, U> SessionUseCaseImpl<UR, RR, PR, UT, RT, A, U> {
    /// Monta o caso de uso.
    pub(crate) fn new(
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
    async fn login(&self, command: LoginCommand) -> Result<Box<dyn User>, AppError> {
        transaction(&self.unit_of_work, async {
            // E-mail desconhecido cai no mesmo erro da senha errada. Distinguir
            // os dois transformaria a tela de login num verificador de quem tem
            // conta no sistema.
            let Some(user) = self.users.find_by_email(&command.email).await? else {
                return Err(AppError::Unauthenticated);
            };

            self.auth_tm.login(user.as_ref(), &command.password)?;

            Ok(user)
        })
        .await
    }

    async fn validate(&self, context: &UserContext) -> Result<Box<dyn User>, AppError> {
        transaction(&self.unit_of_work, async {
            self.users
                .find_by_id(&context.id)
                .await?
                .ok_or(AppError::Unauthenticated)
        })
        .await
    }

    async fn setup(&self, command: SetupCommand) -> Result<Box<dyn User>, AppError> {
        transaction(&self.unit_of_work, async {
            // Um sistema com qualquer usuário já foi montado. Sem esta checagem,
            // o endpoint seria uma porta aberta para criar um administrador a
            // qualquer momento.
            if self.users.has_any().await? {
                return Err(AppError::Conflict(
                    "This system has already been set up.".into(),
                ));
            }

            // As permissões saem do registro preenchido no boot, não de uma
            // lista literal aqui: uma permissão nova é concedida ao
            // administrador sem ninguém lembrar de voltar a este arquivo.
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
