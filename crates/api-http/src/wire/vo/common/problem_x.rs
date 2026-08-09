//! O erro como objeto, para sair pelo mesmo caminho de qualquer resposta.

use crate::wire::dto::json::common::problem_json::ProblemJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;

/// Um problema, na forma da RFC 7807.
///
/// É um VO de resposta como qualquer outro, e é isso que importa: um erro sai
/// pelo `Encoder` da requisição, no formato que o cliente pediu. O desenho
/// anterior tinha um literal de bytes JSON no middleware de pânico e um
/// `application/problem+json` fixo no erro — dois corpos escapando da
/// negociação, num sistema cujo cliente de produção fala `FlatBuffers`.
pub(crate) struct ProblemX {
    /// URI do tipo de problema; `about:blank` quando não há uma página.
    pub(crate) kind: &'static str,
    /// O nome canônico do status.
    pub(crate) title: String,
    /// O status, repetido no corpo para quem só lê o payload.
    pub(crate) status: i32,
    /// O que aconteceu, em texto.
    pub(crate) detail: String,
}

impl ResponseX for ProblemX {
    type Json = ProblemJson;
    type Fbs = fbs::common::ProblemDetails;

    fn to_json(&self) -> Self::Json {
        ProblemJson {
            kind: self.kind.to_owned(),
            title: self.title.clone(),
            status: self.status,
            detail: self.detail.clone(),
            instance: None,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::common::ProblemDetails {
            type_: Some(self.kind.to_owned()),
            title: Some(self.title.clone()),
            status: self.status,
            detail: Some(self.detail.clone()),
            instance: None,
        }
    }
}
