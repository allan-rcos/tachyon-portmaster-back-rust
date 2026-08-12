//! Os testes de `telemetry_event`.

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
