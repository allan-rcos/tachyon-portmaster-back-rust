//! A listagem de contêineres.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::dto::container::container_response_factory::ContainerResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ContainerListView;

/// Monta a tabela da listagem.
pub(crate) struct ContainerListResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: ContainerListView,
}

impl ContainerListResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: ContainerListView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for ContainerListResponseFactory {
    type Table = fbs::container::ContainerListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::ContainerListResponse {
            next_cursor: self.source.next_cursor.clone(),
            data: Some(
                self.source
                    .items
                    .iter()
                    .map(|item| ContainerResponseFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            total: Convert::count(self.source.total),
        })
    }
}
