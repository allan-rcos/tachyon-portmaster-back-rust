//! O DTO de JSON de `TelemetryEvent`.

use serde::{Deserialize, Serialize};

/// O que uma entrada de telemetria registra.
///
/// Sai no fio como o nome da variante, que é o que o `.fbs` publica e o que
/// `swagger/swagger.json` documenta.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum TelemetryEventJson {
    /// Embarque.
    Load,
    /// Desembarque.
    Unload,
}
