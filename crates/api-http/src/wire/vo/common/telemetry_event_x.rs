//! O vocabulário de `TelemetryEvent`, independente de formato.

use crate::wire::dto::json::common::telemetry_event_json::TelemetryEventJson;
use crate::wire::tables as fbs;

/// O que uma entrada de telemetria registra.
///
/// O conjunto é fechado e publicado em `common.fbs`: cliente e servidor
/// compartilham o mesmo vocabulário. Este enum é a forma dele que não
/// depende de formato — os dois DTOs saem daqui, e nenhum deles é este.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TelemetryEventX {
    /// Embarque.
    #[default]
    Load,
    /// Desembarque.
    Unload,
}

impl TelemetryEventX {
    /// O valor a partir do índice que a View carrega.
    ///
    /// Um índice fora da faixa cai no valor neutro em vez de derrubar a
    /// resposta: a linha guardada no banco não é entrada do cliente, e um
    /// registro estranho não deve custar a página inteira a quem a pediu.
    pub(crate) const fn of_index(index: i32) -> Self {
        match index {
            1 => Self::Unload,
            _ => Self::Load,
        }
    }

    /// O valor na tabela do planus.
    pub(crate) const fn to_fbs(self) -> fbs::common::TelemetryEvent {
        match self {
            Self::Load => fbs::common::TelemetryEvent::Load,
            Self::Unload => fbs::common::TelemetryEvent::Unload,
        }
    }

    /// O valor no DTO de JSON.
    pub(crate) const fn to_json(self) -> TelemetryEventJson {
        match self {
            Self::Load => TelemetryEventJson::Load,
            Self::Unload => TelemetryEventJson::Unload,
        }
    }
}
