//! O VO de `ProductCreateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::product::product_create_request_json::ProductCreateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::vo::common::risk_class_x::RiskClassX;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `ProductCreateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProductCreateXRequest {
    /// O nome do produto novo.
    pub(crate) name: Option<String>,
    /// A densidade, que converte quantidade em peso.
    pub(crate) density: Option<f64>,
    /// A classe de risco.
    pub(crate) risk_class: Option<RiskClassX>,
}

impl RequestX for ProductCreateXRequest {
    type Json = ProductCreateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            name: dto.name,
            density: dto.density,
            risk_class: dto.risk_class.map(RiskClassX::of_json),
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::product::ProductCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            name: table.name().ok().map(str::to_owned),
            density: table.density().ok(),
            risk_class: table.risk_class().ok().map(RiskClassX::of_fbs),
        })
    }
}
