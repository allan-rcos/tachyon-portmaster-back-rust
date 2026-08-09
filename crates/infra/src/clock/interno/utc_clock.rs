//! O relógio de parede em UTC. Não sai do crate.

use crate::clock::Clock;
use chrono::{DateTime, Utc};

/// O relógio do sistema operacional, lido em UTC.
#[derive(Debug, Clone, Default)]
pub(crate) struct UtcClock;

impl UtcClock {
    /// Monta o relógio.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl Clock for UtcClock {
    /// A hora do sistema.
    ///
    /// `Utc::now` é o único acesso ao relógio de parede em todo o código, e é o
    /// que os bans de `SystemTime::now` e `Local::now` deixam de pé de
    /// propósito: um ponto só, atrás de uma trait, trocável num teste.
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
