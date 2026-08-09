//! O DTO de JSON de `ContainerListResponse`.

use crate::wire::dto::json::container::container_response_json::ContainerResponseJson;
use serde::Serialize;

/// `ContainerListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ContainerListResponseJson {
    /// A página de contêineres.
    pub(crate) data: Vec<ContainerResponseJson>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos contêineres existem ao todo.
    pub(crate) total: i32,
}
