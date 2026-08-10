//! O VO de `ContainerSummaryListResponse`.

use crate::wire::convert::Convert;
use crate::wire::dto::json::container::container_summary_list_response_json::ContainerSummaryListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::container::container_summary_x_response::ContainerSummaryXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::ContainerSummaryListView;

/// O que a rota de `ContainerSummaryListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ContainerSummaryListXResponse {
    /// A página de resumos.
    pub(crate) data: Vec<ContainerSummaryXResponse>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos resumos existem ao todo.
    pub(crate) total: i32,
}

impl ResponseX for ContainerSummaryListXResponse {
    type Json = ContainerSummaryListResponseJson;
    type Fbs = fbs::container::ContainerSummaryListResponse;

    fn to_json(&self) -> Self::Json {
        ContainerSummaryListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::ContainerSummaryListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }
}

impl ContainerSummaryListXResponse {
    /// A página de resumos.
    pub(crate) fn of(source: ContainerSummaryListView) -> Self {
        Self {
            data: source
                .items
                .into_iter()
                .map(ContainerSummaryXResponse::of)
                .collect(),
            next_cursor: source.next_cursor,
            total: Convert::count(source.total),
        }
    }
}
