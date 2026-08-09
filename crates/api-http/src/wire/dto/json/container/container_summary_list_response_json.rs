//! O DTO de JSON de `ContainerSummaryListResponse`.

use crate::wire::dto::json::container::container_summary_response_json::ContainerSummaryResponseJson;
use serde::Serialize;

/// `ContainerSummaryListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ContainerSummaryListResponseJson {
    /// A página de resumos.
    pub(crate) data: Vec<ContainerSummaryResponseJson>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos resumos existem ao todo.
    pub(crate) total: i32,
}
