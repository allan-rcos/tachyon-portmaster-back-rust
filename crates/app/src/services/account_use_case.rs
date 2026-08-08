//! A conta do próprio usuário.

use crate::commands::account::ChangePasswordCommand;
use crate::commands::account::UpdateAccountCommand;
use crate::error::AppError;
use crate::queries::account::GetAccountQuery;
use portmaster_domain::models::User;
use portmaster_infra::query::views::AccountView;

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
