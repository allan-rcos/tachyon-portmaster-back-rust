//! Os testes de `container_status`.

use super::*;
use pretty_assertions::assert_eq;

/// Os índices das variantes são dado gravado, não detalhe do enum.
///
/// Estes números estão em cada linha do banco. Se este teste quebrar depois
/// de mexer no enum, o problema não é o teste: as linhas já existentes
/// passaram a significar outra coisa.
#[test]
fn indices_das_variantes_sao_estaveis() {
    assert_eq!(ContainerStatus::Empty.as_i32(), 0);
    assert_eq!(ContainerStatus::Loading.as_i32(), 1);
    assert_eq!(ContainerStatus::Sealed.as_i32(), 2);
    assert_eq!(ContainerStatus::InTransit.as_i32(), 3);
}

#[test]
fn indice_desconhecido_nao_vira_variante() {
    assert_eq!(ContainerStatus::from_i32(4), None);
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
