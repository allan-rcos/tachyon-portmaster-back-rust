//! Papéis.

use crate::commands::role::CreateRoleCommand;
use crate::commands::role::UpdateRolePermissionsCommand;
use crate::error::AppError;
use crate::queries::role::GetRoleQuery;
use crate::queries::role::ListRolesQuery;
use portmaster_domain::models::Role;
use portmaster_infra::query::views::{RoleListView, RoleViewItem};

/// O que a apresentação pode pedir sobre papéis.
#[trait_variant::make(Send)]
pub trait RoleUseCase {
    /// Cria e devolve o papel.
    async fn create(&self, command: CreateRoleCommand) -> Result<Box<dyn Role>, AppError>;

    /// Substitui as permissões e devolve o papel atualizado.
    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, AppError>;

    /// Lê um papel.
    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, AppError>;

    /// Lista papéis.
    async fn list(&self, query: ListRolesQuery) -> Result<RoleListView, AppError>;
}
