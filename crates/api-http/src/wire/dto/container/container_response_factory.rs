//! Um contêiner.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ContainerViewItem;

/// Monta a tabela de um contêiner.
pub(crate) struct ContainerResponseFactory {
    source: ContainerViewItem,
}

impl ContainerResponseFactory {
    /// Monta a factory sobre a View, que é o que a leitura devolve.
    pub(crate) const fn of(source: ContainerViewItem) -> Self {
        Self { source }
    }

    /// Monta a factory sobre o objeto de domínio, que é o que a escrita devolve.
    pub(crate) fn of_domain(container: &dyn portmaster_app::domain::Container) -> Self {
        Self {
            source: ContainerViewItem {
                id: container.id().to_owned(),
                code: container.code().to_owned(),
                current_weight: container.current_weight(),
                max_capacity: container.max_capacity(),
                status: container.status().as_i32(),
            },
        }
    }
}

impl ResponseFactory for ContainerResponseFactory {
    type Table = fbs::container::ContainerResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::ContainerResponse {
            id: Some(self.source.id.clone()),
            code: Some(self.source.code.clone()),
            current_weight: self.source.current_weight,
            max_capacity: self.source.max_capacity,
            status: Convert::container_status(self.source.status),
        })
    }
}
