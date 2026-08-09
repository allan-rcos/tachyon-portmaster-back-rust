//! As duas conversões que não são movimento direto entre a origem e o fio.
//!
//! * **Contagens**: as Views usam `i64` (é o que `COUNT(*)` devolve); as tabelas
//!   usam `int32`, porque foi o que o `.fbs` fixou. A conversão **satura** em vez
//!   de truncar — um total absurdo aparece como absurdo, não como negativo.
//! * **Timestamp**: a View guarda epoch em ms; o schema pede `string`. Sai em
//!   RFC 3339, que é o que um cliente lê sem conhecer a convenção de quem
//!   escreveu — e em UTC, como todo horário do sistema.
//!
//! Os enums saíam daqui também. Foram para os próprios VOs
//! (`ContainerStatusX::of_index` e companhia), que é onde o vocabulário mora
//! agora: converter índice em variante é assunto da variante.

use chrono::{DateTime, Utc};

/// Converte um valor de origem no que o VO declara.
pub(crate) struct Convert;

impl Convert {
    /// Uma contagem, saturada na faixa que o wire comporta.
    pub(crate) fn count(value: i64) -> i32 {
        i32::try_from(value).unwrap_or(i32::MAX)
    }

    /// Epoch em ms → RFC 3339, em UTC.
    pub(crate) fn timestamp(epoch_ms: i64) -> Option<String> {
        DateTime::<Utc>::from_timestamp_millis(epoch_ms).map(|at| at.to_rfc3339())
    }
}
