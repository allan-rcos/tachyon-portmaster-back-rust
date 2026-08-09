//! O VO de `ManifestResponse`.

use crate::wire::dto::json::manifest::manifest_response_json::ManifestResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `ManifestResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ManifestXResponse {
    /// O que aconteceu, em texto.
    pub(crate) message: String,
    /// O contêiner depois da movimentação.
    pub(crate) container: ContainerXResponse,
}

impl ResponseX for ManifestXResponse {
    type Json = ManifestResponseJson;
    type Fbs = fbs::manifest::ManifestResponse;

    fn to_json(&self) -> Self::Json {
        ManifestResponseJson {
            message: self.message.clone(),
            container: self.container.to_json(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::manifest::ManifestResponse {
            message: Some(self.message.clone()),
            container: Some(Box::new(self.container.to_fbs())),
        }
    }
}
