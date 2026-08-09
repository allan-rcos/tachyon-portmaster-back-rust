//! O VO de `ContainerResponse`.

use crate::wire::dto::json::container::container_response_json::ContainerResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::common::container_status_x::ContainerStatusX;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::ContainerViewItem;

/// O que a rota de `ContainerResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ContainerXResponse {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// O código do contêiner.
    pub(crate) code: String,
    /// O peso já embarcado.
    pub(crate) current_weight: f64,
    /// A capacidade máxima.
    pub(crate) max_capacity: f64,
    /// Em que ponto do ciclo ele está.
    pub(crate) status: ContainerStatusX,
}

impl ResponseX for ContainerXResponse {
    type Json = ContainerResponseJson;
    type Fbs = fbs::container::ContainerResponse;

    fn to_json(&self) -> Self::Json {
        ContainerResponseJson {
            id: self.id.clone(),
            code: self.code.clone(),
            current_weight: self.current_weight,
            max_capacity: self.max_capacity,
            status: self.status.to_json(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::ContainerResponse {
            id: Some(self.id.clone()),
            code: Some(self.code.clone()),
            current_weight: self.current_weight,
            max_capacity: self.max_capacity,
            status: self.status.to_fbs(),
        }
    }
}

impl ContainerXResponse {
    /// O contêiner, vindo do lado de leitura.
    pub(crate) fn of(source: ContainerViewItem) -> Self {
        Self {
            id: source.id,
            code: source.code,
            current_weight: source.current_weight,
            max_capacity: source.max_capacity,
            status: ContainerStatusX::of_index(source.status),
        }
    }

    /// O contêiner, vindo do objeto de domínio.
    ///
    /// É o caminho da escrita: quem acabou de selar um contêiner tem o objeto em
    /// mãos e não precisa reler a projeção para responder.
    pub(crate) fn of_domain(container: &dyn portmaster_app::domain::Container) -> Self {
        Self {
            id: container.id().to_owned(),
            code: container.code().to_owned(),
            current_weight: container.current_weight(),
            max_capacity: container.max_capacity(),
            status: ContainerStatusX::of_index(container.status().as_i32()),
        }
    }
}
