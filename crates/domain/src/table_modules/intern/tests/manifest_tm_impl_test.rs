//! Os testes de `manifest_tm_impl`.

use super::*;
use crate::enums::RiskClass;
use crate::table_modules::intern::models::product_model::ProductModel;
use pretty_assertions::assert_eq;

/// Contêiner com peso e status arbitrários.
fn container_at(weight: f64, capacity: f64, status: ContainerStatus) -> Box<dyn Container> {
    Box::new(ContainerModel::new(
        "C1".into(),
        "MSCU1234567".into(),
        weight,
        capacity,
        status,
    ))
}

/// Produto de densidade conhecida: 2 kg por unidade.
fn product() -> Box<dyn Product> {
    Box::new(ProductModel::new(
        "P1".into(),
        "Soja".into(),
        2.0,
        RiskClass::None,
    ))
}

/// Linha de manifesto já existente.
fn cargo_of(quantity: f64, weight: f64) -> ManifestCargoModel {
    ManifestCargoModel::new("C1".into(), "P1".into(), quantity, weight)
}

#[test]
fn embarcar_converte_quantidade_em_peso_e_abre_o_carregamento() {
    let container = container_at(0.0, 1000.0, ContainerStatus::Empty);
    let product = product();

    let change = manifest_tm()
        .load(container.as_ref(), product.as_ref(), 10.0, None)
        .expect("10 unidades de 2 kg cabem em 1000 kg");

    assert_eq!(change.container().current_weight(), 20.0);
    assert_eq!(change.container().status(), ContainerStatus::Loading);
    assert_eq!(change.event(), TelemetryEvent::Load);
    assert!(!change.clear_manifest());

    let cargo = change.cargo().expect("a linha do manifesto foi criada");
    assert_eq!(cargo.quantity(), 10.0);
    assert_eq!(cargo.weight(), 20.0);
}

/// Embarcar duas vezes o mesmo produto acumula, não duplica a linha.
#[test]
fn embarcar_soma_ao_que_ja_estava_no_manifesto() {
    let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
    let product = product();
    let existing = cargo_of(10.0, 20.0);

    let change = manifest_tm()
        .load(container.as_ref(), product.as_ref(), 5.0, Some(&existing))
        .expect("cabe");

    let cargo = change.cargo().expect("a linha continua existindo");
    assert_eq!(cargo.quantity(), 15.0);
    assert_eq!(cargo.weight(), 30.0);
    assert_eq!(change.container().current_weight(), 30.0);
}

/// O caso que a tolerância existe para proteger.
///
/// A soma em ponto flutuante pode passar da capacidade por uma fração
/// invisível, e recusar por isso seria incompreensível para quem embarcou.
#[test]
fn a_carga_que_cabe_exatamente_e_aceita() {
    let container = container_at(0.0, 20.0, ContainerStatus::Empty);
    let product = product();

    let change = manifest_tm()
        .load(container.as_ref(), product.as_ref(), 10.0, None)
        .expect("20 kg em 20 kg de capacidade cabe");

    assert_eq!(change.container().current_weight(), 20.0);
}

#[test]
fn recusa_carga_que_nao_cabe() {
    let container = container_at(0.0, 10.0, ContainerStatus::Empty);
    let product = product();

    let error = manifest_tm()
        .load(container.as_ref(), product.as_ref(), 10.0, None)
        .err()
        .expect("20 kg não cabem em 10 kg");

    assert!(matches!(error, ManifestError::ExceedsCapacity));
}

#[test]
fn nao_embarca_em_conteiner_fechado() {
    for status in [ContainerStatus::Sealed, ContainerStatus::InTransit] {
        let container = container_at(100.0, 1000.0, status);
        let product = product();

        let error = manifest_tm()
            .load(container.as_ref(), product.as_ref(), 1.0, None)
            .err()
            .expect("contêiner fechado não recebe carga");

        assert!(
            matches!(error, ManifestError::ContainerClosed),
            "status: {status}"
        );
    }
}

#[test]
fn recusa_quantidade_nao_positiva() {
    let container = container_at(0.0, 1000.0, ContainerStatus::Empty);
    let product = product();

    for bad in [0.0, -1.0, f64::NAN] {
        let error = manifest_tm()
            .load(container.as_ref(), product.as_ref(), bad, None)
            .err()
            .unwrap_or_else(|| panic!("quantidade {bad} deveria ser recusada"));

        assert!(matches!(error, ManifestError::InvalidQuantity));
    }
}

/// Esvaziar o contêiner limpa o manifesto de uma vez.
///
/// Em vez de remover linha a linha, o manifesto inteiro vai junto e o
/// contêiner volta a `Empty`.
#[test]
fn desembarcar_tudo_esvazia_o_conteiner_e_limpa_o_manifesto() {
    let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
    let product = product();
    let existing = cargo_of(10.0, 20.0);

    let change = manifest_tm()
        .unload(container.as_ref(), product.as_ref(), 10.0, Some(&existing))
        .expect("há 10 unidades embarcadas");

    assert_eq!(change.container().current_weight(), 0.0);
    assert_eq!(change.container().status(), ContainerStatus::Empty);
    assert!(change.clear_manifest());
    assert!(change.cargo().is_none());
    assert_eq!(change.event(), TelemetryEvent::Unload);
}

/// O contêiner segue com carga de outro produto: o manifesto **não** é limpo.
///
/// Só a linha do produto zerado desaparece.
#[test]
fn desembarcar_um_produto_por_completo_derruba_so_a_linha_dele() {
    let container = container_at(50.0, 1000.0, ContainerStatus::Loading);
    let product = product();
    let existing = cargo_of(10.0, 20.0);

    let change = manifest_tm()
        .unload(container.as_ref(), product.as_ref(), 10.0, Some(&existing))
        .expect("há 10 unidades embarcadas");

    assert_eq!(change.container().current_weight(), 30.0);
    assert_eq!(change.container().status(), ContainerStatus::Loading);
    assert!(!change.clear_manifest());
    assert!(change.cargo().is_none());
}

#[test]
fn desembarcar_parcialmente_reduz_a_linha() {
    let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
    let product = product();
    let existing = cargo_of(10.0, 20.0);

    let change = manifest_tm()
        .unload(container.as_ref(), product.as_ref(), 4.0, Some(&existing))
        .expect("há o bastante embarcado");

    let cargo = change.cargo().expect("a linha continua existindo");
    assert_eq!(cargo.quantity(), 6.0);
    assert_eq!(cargo.weight(), 12.0);
    assert_eq!(change.container().current_weight(), 12.0);
}

#[test]
fn recusa_desembarcar_mais_do_que_esta_embarcado() {
    let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
    let product = product();
    let existing = cargo_of(10.0, 20.0);

    let error = manifest_tm()
        .unload(container.as_ref(), product.as_ref(), 11.0, Some(&existing))
        .err()
        .expect("não há 11 unidades");

    assert!(matches!(error, ManifestError::InsufficientCargo));
}

#[test]
fn recusa_desembarcar_o_que_nunca_foi_embarcado() {
    let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
    let product = product();

    let error = manifest_tm()
        .unload(container.as_ref(), product.as_ref(), 1.0, None)
        .err()
        .expect("sem linha de manifesto não há o que tirar");

    assert!(matches!(error, ManifestError::InsufficientCargo));
}

#[test]
fn so_desembarca_o_que_esta_carregando() {
    for status in [
        ContainerStatus::Empty,
        ContainerStatus::Sealed,
        ContainerStatus::InTransit,
    ] {
        let container = container_at(20.0, 1000.0, status);
        let product = product();
        let existing = cargo_of(10.0, 20.0);

        let error = manifest_tm()
            .unload(container.as_ref(), product.as_ref(), 1.0, Some(&existing))
            .err()
            .unwrap_or_else(|| panic!("{status} não pode ser descarregado"));

        assert!(
            matches!(error, ManifestError::UnloadRequiresLoading),
            "status: {status}"
        );
    }
}
