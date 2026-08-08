//! A listagem de resumos de contêiner.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::dto::container::container_summary_response_factory::ContainerSummaryResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ContainerSummaryListView;

/// Monta a tabela da listagem.
pub(crate) struct ContainerSummaryListResponseFactory {
    source: ContainerSummaryListView,
}

impl ContainerSummaryListResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: ContainerSummaryListView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for ContainerSummaryListResponseFactory {
    type Table = fbs::container::ContainerSummaryListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::ContainerSummaryListResponse {
            next_cursor: self.source.next_cursor.clone(),
            data: Some(
                self.source
                    .items
                    .iter()
                    .map(|item| ContainerSummaryResponseFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            total: Convert::count(self.source.total),
        })
    }
}
