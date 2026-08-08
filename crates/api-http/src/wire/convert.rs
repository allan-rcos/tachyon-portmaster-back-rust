//! As conversões que não são movimento direto entre a origem e o fio.
//!
//! Três lugares onde os tipos não batem, e os três moram aqui em vez de
//! espalhados pelas factories:
//!
//! * **Contagens**: as Views usam `i64` (é o que `COUNT(*)` devolve); as tabelas
//!   usam `int32`, porque foi o que o `.fbs` fixou. A conversão **satura** em vez
//!   de truncar — um total absurdo aparece como absurdo, não como negativo.
//! * **Enums**: a View carrega o índice; o wire tem o seu próprio enum. Um
//!   índice fora da faixa cai no valor neutro em vez de derrubar a resposta.
//! * **Timestamp**: a View guarda epoch em ms; o schema pede `string`. Sai em
//!   RFC 3339, que é o que um cliente lê sem conhecer a convenção de quem
//!   escreveu — e em UTC, como todo horário do sistema.

use chrono::{DateTime, Utc};

use crate::wire::tables as fbs;

/// Converte um valor de origem no que a tabela do wire declara.
pub(crate) struct Convert;

impl Convert {
    /// Uma contagem, saturada na faixa que o wire comporta.
    pub(crate) fn count(value: i64) -> i32 {
        i32::try_from(value).unwrap_or(i32::MAX)
    }

    /// O índice de classe de risco, no enum do wire.
    pub(crate) fn risk_class(index: i32) -> fbs::common::RiskClass {
        u8::try_from(index)
            .ok()
            .and_then(|index| fbs::common::RiskClass::try_from(index).ok())
            .unwrap_or(fbs::common::RiskClass::None)
    }

    /// O índice de status, no enum do wire.
    pub(crate) fn container_status(index: i32) -> fbs::common::ContainerStatus {
        u8::try_from(index)
            .ok()
            .and_then(|index| fbs::common::ContainerStatus::try_from(index).ok())
            .unwrap_or(fbs::common::ContainerStatus::Empty)
    }

    /// O índice de evento, no enum do wire.
    pub(crate) fn telemetry_event(index: i32) -> fbs::common::TelemetryEvent {
        u8::try_from(index)
            .ok()
            .and_then(|index| fbs::common::TelemetryEvent::try_from(index).ok())
            .unwrap_or(fbs::common::TelemetryEvent::Load)
    }

    /// Epoch em ms → RFC 3339, em UTC.
    pub(crate) fn timestamp(epoch_ms: i64) -> Option<String> {
        DateTime::<Utc>::from_timestamp_millis(epoch_ms).map(|at| at.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Um `COUNT(*)` acima de 2^31 é absurdo, mas truncá-lo produziria um
    /// negativo — que o cliente exibiria como se fosse um dado.
    #[test]
    fn a_contagem_satura_em_vez_de_truncar() {
        assert_eq!(Convert::count(i64::from(i32::MAX) + 1), i32::MAX);
        assert_eq!(Convert::count(7), 7);
    }

    #[test]
    fn um_indice_de_enum_fora_da_faixa_cai_no_neutro() {
        assert_eq!(Convert::risk_class(99), fbs::common::RiskClass::None);
        assert_eq!(
            Convert::container_status(-1),
            fbs::common::ContainerStatus::Empty
        );
        assert_eq!(
            Convert::risk_class(2),
            fbs::common::RiskClass::Class3FlammableLiquids
        );
    }

    #[test]
    fn o_timestamp_sai_em_utc_e_interpretavel() {
        assert_eq!(
            Convert::timestamp(0).as_deref(),
            Some("1970-01-01T00:00:00+00:00")
        );
    }
}
