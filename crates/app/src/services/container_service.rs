//! Contêineres.

use crate::commands::container::ContainerCommand;
use crate::commands::container::CreateContainerCommand;
use crate::commands::container::UpdateContainerCommand;
use crate::error::ContainerError;
use crate::queries::container::GetContainerQuery;
use crate::queries::container::ListContainerSummariesQuery;
use crate::queries::container::ListContainersQuery;
use crate::services::MetadataService;
use portmaster_domain::domain::Container;
use portmaster_infra::query::views::{
    ContainerListView, ContainerSummaryListView, ContainerViewItem,
};

/// O que a apresentação pode pedir sobre contêineres.
#[trait_variant::make(Send)]
pub trait ContainerService {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ContainerError>;

    /// Registra e devolve o contêiner criado.
    async fn create(
        &self,
        command: CreateContainerCommand,
    ) -> Result<Box<dyn Container>, ContainerError>;

    /// Altera a capacidade e devolve o contêiner atualizado.
    async fn update(
        &self,
        command: UpdateContainerCommand,
    ) -> Result<Box<dyn Container>, ContainerError>;

    /// Remove — soft-delete.
    async fn delete(&self, command: ContainerCommand) -> Result<(), ContainerError>;

    /// Sela: exige estar carregando e ter ao menos 10% da capacidade.
    async fn seal(&self, command: ContainerCommand) -> Result<(), ContainerError>;

    /// Despacha: exige estar selado, e é o que impede despachar duas vezes.
    async fn dispatch(&self, command: ContainerCommand) -> Result<(), ContainerError>;

    /// Lê um contêiner.
    async fn get(&self, query: GetContainerQuery) -> Result<ContainerViewItem, ContainerError>;

    /// Lista contêineres.
    async fn list(&self, query: ListContainersQuery) -> Result<ContainerListView, ContainerError>;

    /// Lista contêineres com carga e telemetria recente.
    async fn list_summaries(
        &self,
        query: ListContainerSummariesQuery,
    ) -> Result<ContainerSummaryListView, ContainerError>;
}
