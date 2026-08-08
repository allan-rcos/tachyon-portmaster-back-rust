//! Contêineres.

use crate::commands::container::ContainerCommand;
use crate::commands::container::CreateContainerCommand;
use crate::commands::container::UpdateContainerCommand;
use crate::error::AppError;
use crate::queries::container::GetContainerQuery;
use crate::queries::container::ListContainerSummariesQuery;
use crate::queries::container::ListContainersQuery;
use portmaster_domain::models::Container;
use portmaster_infra::query::views::{
    ContainerListView, ContainerSummaryListView, ContainerViewItem,
};

/// O que a apresentação pode pedir sobre contêineres.
#[trait_variant::make(Send)]
pub trait ContainerUseCase {
    /// Registra e devolve o contêiner criado.
    async fn create(&self, command: CreateContainerCommand)
        -> Result<Box<dyn Container>, AppError>;

    /// Altera a capacidade e devolve o contêiner atualizado.
    async fn update(&self, command: UpdateContainerCommand)
        -> Result<Box<dyn Container>, AppError>;

    /// Remove — soft-delete.
    async fn delete(&self, command: ContainerCommand) -> Result<(), AppError>;

    /// Sela: exige estar carregando e ter ao menos 10% da capacidade.
    async fn seal(&self, command: ContainerCommand) -> Result<(), AppError>;

    /// Despacha: exige estar selado, e é o que impede despachar duas vezes.
    async fn dispatch(&self, command: ContainerCommand) -> Result<(), AppError>;

    /// Lê um contêiner.
    async fn get(&self, query: GetContainerQuery) -> Result<ContainerViewItem, AppError>;

    /// Lista contêineres.
    async fn list(&self, query: ListContainersQuery) -> Result<ContainerListView, AppError>;

    /// Lista contêineres com carga e telemetria recente.
    async fn list_summaries(
        &self,
        query: ListContainerSummariesQuery,
    ) -> Result<ContainerSummaryListView, AppError>;
}
