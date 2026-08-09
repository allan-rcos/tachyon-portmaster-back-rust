//! O DTO de JSON de `RiskClass`.

use serde::{Deserialize, Serialize};

/// A classe de risco de um produto, na numeração da ONU.
///
/// Sai no fio como o nome da variante, que é o que o `.fbs` publica e o que
/// `swagger/swagger.json` documenta.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum RiskClassJson {
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
    None,
}
