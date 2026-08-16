//! Onde um contêiner está no seu ciclo de vida.

use std::fmt;

use crate::enums::unknown_index::UnknownIndex;

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
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor que não corresponda a
    /// nenhuma variante em vez de escolher uma por aproximação.
    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Empty),
            1 => Some(Self::Loading),
            2 => Some(Self::Sealed),
            3 => Some(Self::InTransit),
            _ => None,
        }
    }
}

impl TryFrom<i32> for ContainerStatus {
    type Error = UnknownIndex;

    /// O mesmo que [`from_i32`](Self::from_i32), com a recusa já explicada.
    ///
    /// É esta a forma que a leitura de coluna das entities usa: a mensagem sai
    /// daqui, de onde se sabe qual enum recusou, e não do ponto de leitura.
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or_else(|| UnknownIndex::new(value, "ContainerStatus"))
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
