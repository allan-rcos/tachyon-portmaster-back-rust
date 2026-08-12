//! Os testes de `container_tm_impl`.

use super::*;
use crate::table_modules::intern::helpers::fields_of::fields_of;
use pretty_assertions::assert_eq;

struct FixedIdGenerator;
impl DatabaseIdGenerator for FixedIdGenerator {
    fn next(&self) -> String {
        "C1".to_string()
    }
}

fn table_module() -> ContainerTMImpl<FixedIdGenerator> {
    ContainerTMImpl::new(FixedIdGenerator)
}

/// Contêiner com peso e status arbitrários, para exercitar as transições.
fn container_at(weight: f64, capacity: f64, status: ContainerStatus) -> Box<dyn Container> {
    Box::new(ContainerModel::new(
        "C1".into(),
        "MSCU1234567".into(),
        weight,
        capacity,
        status,
    ))
}

#[test]
fn nasce_vazio_e_sem_peso() {
    let container = table_module()
        .create("MSCU1234567".into(), 1000.0)
        .expect("os dados são válidos");

    assert_eq!(container.status(), ContainerStatus::Empty);
    assert_eq!(container.current_weight(), 0.0);
    assert_eq!(container.max_capacity(), 1000.0);
}

#[test]
fn recusa_codigo_e_capacidade_invalidos() {
    let error = table_module()
        .create(String::new(), 0.0)
        .err()
        .expect("os dois campos são inválidos");

    let ContainerError::Validation(fields) = error else {
        panic!("esperava erro de validação");
    };
    assert_eq!(fields_of(&fields), vec!["code", "max_capacity"]);
}

#[test]
fn sela_um_conteiner_carregado_o_bastante() {
    let container = container_at(100.0, 1000.0, ContainerStatus::Loading);
    let sealed = table_module()
        .seal(container.as_ref())
        .expect("10% da capacidade é exatamente o mínimo");

    assert_eq!(sealed.status(), ContainerStatus::Sealed);
    // A transição não tocou o original.
    assert_eq!(container.status(), ContainerStatus::Loading);
}

#[test]
fn recusa_selar_abaixo_do_minimo() {
    let container = container_at(99.9, 1000.0, ContainerStatus::Loading);
    let error = table_module()
        .seal(container.as_ref())
        .err()
        .expect("abaixo de 10% não sela");

    assert!(matches!(error, ContainerError::SealBelowMinimumFill));
}

#[test]
fn so_sela_o_que_esta_carregando() {
    for status in [
        ContainerStatus::Empty,
        ContainerStatus::Sealed,
        ContainerStatus::InTransit,
    ] {
        let container = container_at(500.0, 1000.0, status);
        let error = table_module()
            .seal(container.as_ref())
            .err()
            .unwrap_or_else(|| panic!("{status} não pode ser selado"));

        assert!(
            matches!(error, ContainerError::SealRequiresLoading),
            "status: {status}"
        );
    }
}

#[test]
fn despacha_o_que_esta_selado() {
    let container = container_at(500.0, 1000.0, ContainerStatus::Sealed);
    let dispatched = table_module()
        .dispatch(container.as_ref())
        .expect("selado pode ser despachado");

    assert_eq!(dispatched.status(), ContainerStatus::InTransit);
}

/// Depois do primeiro despacho o contêiner não está mais `Sealed`.
///
/// É o que impede despachar duas vezes.
#[test]
fn o_segundo_despacho_e_recusado() {
    let dispatched = container_at(500.0, 1000.0, ContainerStatus::InTransit);
    let error = table_module()
        .dispatch(dispatched.as_ref())
        .err()
        .expect("já despachado não despacha de novo");

    assert!(matches!(error, ContainerError::DispatchRequiresSealed));
}
