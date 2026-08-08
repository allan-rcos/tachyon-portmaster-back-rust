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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn indices_das_variantes_sao_estaveis() {
        assert_eq!(TelemetryEvent::Load.as_i32(), 0);
        assert_eq!(TelemetryEvent::Unload.as_i32(), 1);
    }

    #[test]
    fn indice_desconhecido_nao_vira_variante() {
        assert_eq!(TelemetryEvent::from_i32(2), None);
    }
}
