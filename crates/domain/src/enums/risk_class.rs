//! A classe de risco de um produto, na numeração da ONU.

use crate::enums::unknown_index::UnknownIndex;

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
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor desconhecido.
    pub const fn from_i32(value: i32) -> Option<Self> {
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

impl TryFrom<i32> for RiskClass {
    type Error = UnknownIndex;

    /// O mesmo que [`from_i32`](Self::from_i32), com a recusa já explicada.
    ///
    /// É esta a forma que o `#[sqlx(try_from = "i32")]` das entities usa: a
    /// mensagem sai daqui, de onde se sabe qual enum recusou, e não do ponto de
    /// leitura.
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or_else(|| UnknownIndex::new(value, "RiskClass"))
    }
}
