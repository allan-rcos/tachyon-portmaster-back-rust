//! O DTO de JSON de um problema.

use serde::Serialize;

/// O corpo de erro em JSON, na ordem que `ProblemDetails` do `common.fbs` fixa.
#[derive(Debug, Serialize)]
pub(crate) struct ProblemJson {
    /// URI do tipo de problema.
    ///
    /// `type` é palavra reservada em Rust; o `rename` devolve o nome que a RFC
    /// 7807 exige no fio.
    #[serde(rename = "type")]
    pub(crate) kind: String,
    /// O nome canônico do status.
    pub(crate) title: String,
    /// O status, repetido no corpo.
    pub(crate) status: i32,
    /// O que aconteceu, em texto.
    pub(crate) detail: String,
    /// A URI do caso concreto, quando há uma.
    pub(crate) instance: Option<String>,
}
