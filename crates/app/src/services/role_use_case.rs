//! Papéis.

use crate::commands::role::CreateRoleCommand;
use crate::commands::role::UpdateRolePermissionsCommand;
use crate::error::RoleError;
use crate::queries::role::GetRoleQuery;
use crate::queries::role::ListRolesQuery;
use crate::services::MetadataUseCase;
use portmaster_domain::domain::Role;
use portmaster_infra::query::views::{RoleListView, RoleViewItem};

/// O que a apresentação pode pedir sobre papéis.
#[trait_variant::make(Send)]
pub trait RoleUseCase {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataUseCase + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), RoleError>;

    /// Cria e devolve o papel.
    async fn create(&self, command: CreateRoleCommand) -> Result<Box<dyn Role>, RoleError>;

    /// Substitui as permissões e devolve o papel atualizado.
    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, RoleError>;

    /// Lê um papel.
    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, RoleError>;

    /// Lista papéis.
    async fn list(&self, query: ListRolesQuery) -> Result<RoleListView, RoleError>;
}
