//! O VO de `ContainerListResponse`.

use crate::wire::dto::json::container::container_list_response_json::ContainerListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use crate::wire::x::response_x::ResponseX;
use crate::wire::convert::Convert;
use portmaster_app::views::ContainerListView;

/// O que a rota de `ContainerListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ContainerListXResponse {
    /// A página de contêineres.
    pub(crate) data: Vec<ContainerXResponse>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos contêineres existem ao todo.
    pub(crate) total: i32,
}

impl ResponseX for ContainerListXResponse {
    type Json = ContainerListResponseJson;
    type Fbs = fbs::container::ContainerListResponse;

    fn to_json(&self) -> Self::Json {
        ContainerListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::ContainerListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }
}

impl ContainerListXResponse {
    /// A página de contêineres.
    pub(crate) fn of(source: ContainerListView) -> Self {
        Self {
            data: source.items.into_iter().map(ContainerXResponse::of).collect(),
            next_cursor: source.next_cursor,
            total: Convert::count(source.total),
        }
    }
}
