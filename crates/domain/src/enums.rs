//! Valores imutáveis que mudam a regra de negócio.
//!
//! Um valor que o domínio nunca altera e que decide o comportamento é um enum —
//! não uma linha de banco nem uma string solta. Fechá-lo no código é o ponto:
//! um `match` sobre o status obriga o compilador a apontar cada lugar que
//! esqueceu de tratar um estado novo, o que uma string jamais faria.
//!
//! A representação numérica de cada variante é o que vai ao banco e ao fio. Os
//! valores são fixos e **não podem ser reordenados** — mudar o índice de uma
//! variante reinterpreta silenciosamente as linhas já gravadas.

use std::fmt;

/// Onde um contêiner está no seu ciclo de vida no pátio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ContainerStatus {
    /// Registrado e sem carga.
    Empty = 0,
    /// Recebendo carga; ainda aceita embarque e desembarque.
    Loading = 1,
    /// Fechado para carga, aguardando despacho.
    Sealed = 2,
    /// Despachado. Estado final.
    InTransit = 3,
}

impl ContainerStatus {
    /// O índice gravado no banco e no fio.
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor que não corresponda a
    /// nenhuma variante em vez de escolher uma por aproximação.
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Empty),
            1 => Some(Self::Loading),
            2 => Some(Self::Sealed),
            3 => Some(Self::InTransit),
            _ => None,
        }
    }
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Empty => "Empty",
            Self::Loading => "Loading",
            Self::Sealed => "Sealed",
            Self::InTransit => "InTransit",
        };
        f.write_str(name)
    }
}

/// Classe de risco do produto, na numeração das Nações Unidas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RiskClass {
    /// Classe 1 — explosivos.
    Class1Explosives = 0,
    /// Classe 2 — gases.
    Class2Gases = 1,
    /// Classe 3 — líquidos inflamáveis.
    Class3FlammableLiquids = 2,
    /// Classe 4 — sólidos inflamáveis.
    Class4FlammableSolids = 3,
    /// Classe 5 — substâncias oxidantes.
    Class5OxidizingSubstances = 4,
    /// Classe 6 — substâncias tóxicas.
    Class6ToxicSubstances = 5,
    /// Classe 7 — materiais radioativos.
    Class7RadioactiveMaterials = 6,
    /// Classe 8 — substâncias corrosivas.
    Class8CorrosiveSubstances = 7,
    /// Classe 9 — diversos.
    Class9Miscellaneous = 8,
    /// Carga sem classificação de risco.
    None = 9,
}

impl RiskClass {
    /// O índice gravado no banco e no fio.
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor desconhecido.
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Class1Explosives),
            1 => Some(Self::Class2Gases),
            2 => Some(Self::Class3FlammableLiquids),
            3 => Some(Self::Class4FlammableSolids),
            4 => Some(Self::Class5OxidizingSubstances),
            5 => Some(Self::Class6ToxicSubstances),
            6 => Some(Self::Class7RadioactiveMaterials),
            7 => Some(Self::Class8CorrosiveSubstances),
            8 => Some(Self::Class9Miscellaneous),
            9 => Some(Self::None),
            _ => None,
        }
    }
}

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
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor desconhecido.
    pub fn from_i32(value: i32) -> Option<Self> {
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
        // Estes números estão gravados em cada linha do banco. Se um teste aqui
        // quebrar depois de mexer no enum, o problema não é o teste: as linhas
        // já existentes passaram a significar outra coisa.
        assert_eq!(ContainerStatus::Empty.as_i32(), 0);
        assert_eq!(ContainerStatus::Loading.as_i32(), 1);
        assert_eq!(ContainerStatus::Sealed.as_i32(), 2);
        assert_eq!(ContainerStatus::InTransit.as_i32(), 3);
        assert_eq!(RiskClass::Class1Explosives.as_i32(), 0);
        assert_eq!(RiskClass::None.as_i32(), 9);
        assert_eq!(TelemetryEvent::Load.as_i32(), 0);
        assert_eq!(TelemetryEvent::Unload.as_i32(), 1);
    }

    #[test]
    fn indice_desconhecido_nao_vira_variante() {
        assert_eq!(ContainerStatus::from_i32(4), None);
        assert_eq!(RiskClass::from_i32(10), None);
        assert_eq!(TelemetryEvent::from_i32(2), None);
        assert_eq!(ContainerStatus::from_i32(-1), None);
    }

    #[test]
    fn ida_e_volta_preserva_a_variante() {
        for status in [
            ContainerStatus::Empty,
            ContainerStatus::Loading,
            ContainerStatus::Sealed,
            ContainerStatus::InTransit,
        ] {
            assert_eq!(ContainerStatus::from_i32(status.as_i32()), Some(status));
        }
    }
}
