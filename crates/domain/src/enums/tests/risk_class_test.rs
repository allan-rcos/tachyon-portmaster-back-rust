//! Os testes de `risk_class`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn indices_das_variantes_sao_estaveis() {
    assert_eq!(RiskClass::Class1Explosives.as_i32(), 0);
    assert_eq!(RiskClass::None.as_i32(), 9);
}

#[test]
fn indice_desconhecido_nao_vira_variante() {
    assert_eq!(RiskClass::from_i32(10), None);
}
