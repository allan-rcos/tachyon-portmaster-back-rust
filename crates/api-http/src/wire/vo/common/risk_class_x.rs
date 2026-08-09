//! O vocabulário de `RiskClass`, independente de formato.

use crate::wire::dto::json::common::risk_class_json::RiskClassJson;
use crate::wire::tables as fbs;

/// A classe de risco de um produto, na numeração da ONU.
///
/// O conjunto é fechado e publicado em `common.fbs`: cliente e servidor
/// compartilham o mesmo vocabulário. Este enum é a forma dele que não
/// depende de formato — os dois DTOs saem daqui, e nenhum deles é este.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RiskClassX {
    /// Explosivos.
    Class1Explosives,
    /// Gases.
    Class2Gases,
    /// Líquidos inflamáveis.
    Class3FlammableLiquids,
    /// Sólidos inflamáveis.
    Class4FlammableSolids,
    /// Substâncias oxidantes.
    Class5OxidizingSubstances,
    /// Substâncias tóxicas.
    Class6ToxicSubstances,
    /// Materiais radioativos.
    Class7RadioactiveMaterials,
    /// Substâncias corrosivas.
    Class8CorrosiveSubstances,
    /// Diversos.
    Class9Miscellaneous,
    /// Sem classificação.
    #[default]
    None,
}

impl RiskClassX {
    /// O valor a partir do índice que a View carrega.
    ///
    /// Um índice fora da faixa cai no valor neutro em vez de derrubar a
    /// resposta: a linha guardada no banco não é entrada do cliente, e um
    /// registro estranho não deve custar a página inteira a quem a pediu.
    pub(crate) const fn of_index(index: i32) -> Self {
        match index {
            0 => Self::Class1Explosives,
            1 => Self::Class2Gases,
            2 => Self::Class3FlammableLiquids,
            3 => Self::Class4FlammableSolids,
            4 => Self::Class5OxidizingSubstances,
            5 => Self::Class6ToxicSubstances,
            6 => Self::Class7RadioactiveMaterials,
            7 => Self::Class8CorrosiveSubstances,
            8 => Self::Class9Miscellaneous,
            _ => Self::None,
        }
    }

    /// O índice que o domínio usa.
    pub(crate) const fn as_index(self) -> i32 {
        match self {
            Self::Class1Explosives => 0,
            Self::Class2Gases => 1,
            Self::Class3FlammableLiquids => 2,
            Self::Class4FlammableSolids => 3,
            Self::Class5OxidizingSubstances => 4,
            Self::Class6ToxicSubstances => 5,
            Self::Class7RadioactiveMaterials => 6,
            Self::Class8CorrosiveSubstances => 7,
            Self::Class9Miscellaneous => 8,
            Self::None => 9,
        }
    }

    /// O valor a partir da tabela do planus.
    pub(crate) const fn of_fbs(value: fbs::common::RiskClass) -> Self {
        match value {
            fbs::common::RiskClass::Class1Explosives => Self::Class1Explosives,
            fbs::common::RiskClass::Class2Gases => Self::Class2Gases,
            fbs::common::RiskClass::Class3FlammableLiquids => Self::Class3FlammableLiquids,
            fbs::common::RiskClass::Class4FlammableSolids => Self::Class4FlammableSolids,
            fbs::common::RiskClass::Class5OxidizingSubstances => Self::Class5OxidizingSubstances,
            fbs::common::RiskClass::Class6ToxicSubstances => Self::Class6ToxicSubstances,
            fbs::common::RiskClass::Class7RadioactiveMaterials => Self::Class7RadioactiveMaterials,
            fbs::common::RiskClass::Class8CorrosiveSubstances => Self::Class8CorrosiveSubstances,
            fbs::common::RiskClass::Class9Miscellaneous => Self::Class9Miscellaneous,
            fbs::common::RiskClass::None => Self::None,
        }
    }

    /// O valor a partir do DTO de JSON.
    pub(crate) const fn of_json(value: RiskClassJson) -> Self {
        match value {
            RiskClassJson::Class1Explosives => Self::Class1Explosives,
            RiskClassJson::Class2Gases => Self::Class2Gases,
            RiskClassJson::Class3FlammableLiquids => Self::Class3FlammableLiquids,
            RiskClassJson::Class4FlammableSolids => Self::Class4FlammableSolids,
            RiskClassJson::Class5OxidizingSubstances => Self::Class5OxidizingSubstances,
            RiskClassJson::Class6ToxicSubstances => Self::Class6ToxicSubstances,
            RiskClassJson::Class7RadioactiveMaterials => Self::Class7RadioactiveMaterials,
            RiskClassJson::Class8CorrosiveSubstances => Self::Class8CorrosiveSubstances,
            RiskClassJson::Class9Miscellaneous => Self::Class9Miscellaneous,
            RiskClassJson::None => Self::None,
        }
    }

    /// O valor na tabela do planus.
    pub(crate) const fn to_fbs(self) -> fbs::common::RiskClass {
        match self {
            Self::Class1Explosives => fbs::common::RiskClass::Class1Explosives,
            Self::Class2Gases => fbs::common::RiskClass::Class2Gases,
            Self::Class3FlammableLiquids => fbs::common::RiskClass::Class3FlammableLiquids,
            Self::Class4FlammableSolids => fbs::common::RiskClass::Class4FlammableSolids,
            Self::Class5OxidizingSubstances => fbs::common::RiskClass::Class5OxidizingSubstances,
            Self::Class6ToxicSubstances => fbs::common::RiskClass::Class6ToxicSubstances,
            Self::Class7RadioactiveMaterials => fbs::common::RiskClass::Class7RadioactiveMaterials,
            Self::Class8CorrosiveSubstances => fbs::common::RiskClass::Class8CorrosiveSubstances,
            Self::Class9Miscellaneous => fbs::common::RiskClass::Class9Miscellaneous,
            Self::None => fbs::common::RiskClass::None,
        }
    }

    /// O valor no DTO de JSON.
    pub(crate) const fn to_json(self) -> RiskClassJson {
        match self {
            Self::Class1Explosives => RiskClassJson::Class1Explosives,
            Self::Class2Gases => RiskClassJson::Class2Gases,
            Self::Class3FlammableLiquids => RiskClassJson::Class3FlammableLiquids,
            Self::Class4FlammableSolids => RiskClassJson::Class4FlammableSolids,
            Self::Class5OxidizingSubstances => RiskClassJson::Class5OxidizingSubstances,
            Self::Class6ToxicSubstances => RiskClassJson::Class6ToxicSubstances,
            Self::Class7RadioactiveMaterials => RiskClassJson::Class7RadioactiveMaterials,
            Self::Class8CorrosiveSubstances => RiskClassJson::Class8CorrosiveSubstances,
            Self::Class9Miscellaneous => RiskClassJson::Class9Miscellaneous,
            Self::None => RiskClassJson::None,
        }
    }
}
