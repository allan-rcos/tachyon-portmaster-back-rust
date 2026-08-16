//! O que um registro de telemetria diz que aconteceu.

/// O que um registro de telemetria diz que aconteceu com um contêiner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TelemetryEvent {
    /// Carga embarcada.
    Load = 0,
    /// Carga desembarcada.
    Unload = 1,
}

impl TelemetryEvent {
    /// O índice gravado no banco e no fio.
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor desconhecido.
    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Load),
            1 => Some(Self::Unload),
            _ => None,
        }
    }
}
