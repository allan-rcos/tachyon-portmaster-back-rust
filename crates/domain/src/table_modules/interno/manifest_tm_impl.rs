//! A implementação das regras de manifesto.

use crate::enums::{ContainerStatus, TelemetryEvent};
use crate::error::ManifestError;
use crate::models::interno::container_model::ContainerModel;
use crate::models::interno::manifest_cargo_model::ManifestCargoModel;
use crate::models::interno::manifest_change_model::ManifestChangeModel;
use crate::models::{Container, ManifestCargo, ManifestChange, Product};
use crate::table_modules::ManifestTM;

/// A implementação. Não precisa de helper nenhum: é aritmética e regra pura.
pub(crate) struct ManifestTMImpl;

impl ManifestTMImpl {
    /// Monta o `TableModule`.
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Produz o contêiner com outro peso e status.
    fn with_weight_and_status(
        container: &dyn Container,
        weight: f64,
        status: ContainerStatus,
    ) -> Box<dyn Container> {
        let mut model = ContainerModel::from_domain(container);
        model.set_weight_and_status(weight, status);
        Box::new(model)
    }
}

impl ManifestTM for ManifestTMImpl {
    fn load(
        &self,
        container: &dyn Container,
        product: &dyn Product,
        quantity: f64,
        current: Option<&dyn ManifestCargo>,
    ) -> Result<Box<dyn ManifestChange>, ManifestError> {
        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(ManifestError::InvalidQuantity);
        }

        if matches!(
            container.status(),
            ContainerStatus::Sealed | ContainerStatus::InTransit
        ) {
            return Err(ManifestError::ContainerClosed);
        }

        let item_weight = product.density() * quantity;
        let new_container_weight = container.current_weight() + item_weight;

        if new_container_weight > container.max_capacity() + EPSILON {
            return Err(ManifestError::ExceedsCapacity);
        }

        // Soma ao que já estava embarcado daquele produto, se havia.
        let existing_quantity = current.map_or(0.0, ManifestCargo::quantity);
        let existing_weight = current.map_or(0.0, ManifestCargo::weight);

        let cargo = ManifestCargoModel::new(
            container.id().to_owned(),
            product.id().to_owned(),
            existing_quantity + quantity,
            existing_weight + item_weight,
        );

        Ok(Box::new(ManifestChangeModel::new(
            Self::with_weight_and_status(container, new_container_weight, ContainerStatus::Loading),
            product.id().to_owned(),
            Some(Box::new(cargo)),
            false,
            TelemetryEvent::Load,
        )))
    }

    fn unload(
        &self,
        container: &dyn Container,
        product: &dyn Product,
        quantity: f64,
        current: Option<&dyn ManifestCargo>,
    ) -> Result<Box<dyn ManifestChange>, ManifestError> {
        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(ManifestError::InvalidQuantity);
        }

        if container.status() != ContainerStatus::Loading {
            return Err(ManifestError::UnloadRequiresLoading);
        }

        // Sem linha de manifesto, ou com menos do que se pediu: não há o que
        // tirar. A tolerância deixa passar o caso de descarregar exatamente
        // tudo que está lá.
        let Some(current) = current else {
            return Err(ManifestError::InsufficientCargo);
        };
        if current.quantity() + EPSILON < quantity {
            return Err(ManifestError::InsufficientCargo);
        }

        let item_weight = product.density() * quantity;
        let new_container_weight = (container.current_weight() - item_weight).max(0.0);
        let new_cargo_quantity = current.quantity() - quantity;
        let new_cargo_weight = (current.weight() - item_weight).max(0.0);

        // Contêiner esvaziou: volta a `Empty` e o manifesto inteiro vai junto.
        if new_container_weight <= EPSILON {
            return Ok(Box::new(ManifestChangeModel::new(
                Self::with_weight_and_status(container, 0.0, ContainerStatus::Empty),
                product.id().to_owned(),
                None,
                true,
                TelemetryEvent::Unload,
            )));
        }

        // Este produto saiu por completo, mas o contêiner segue com carga: só a
        // linha dele desaparece.
        let cargo: Option<Box<dyn ManifestCargo>> = if new_cargo_quantity <= EPSILON {
            None
        } else {
            Some(Box::new(ManifestCargoModel::new(
                container.id().to_owned(),
                product.id().to_owned(),
                new_cargo_quantity,
                new_cargo_weight,
            )))
        };

        Ok(Box::new(ManifestChangeModel::new(
            Self::with_weight_and_status(container, new_container_weight, ContainerStatus::Loading),
            product.id().to_owned(),
            cargo,
            false,
            TelemetryEvent::Unload,
        )))
    }
}

/// Tolerância nas comparações de peso e quantidade.
const EPSILON: f64 = 0.000_000_1;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::RiskClass;
    use crate::models::interno::product_model::ProductModel;
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

        let change = ManifestTMImpl::new()
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

    #[test]
    fn embarcar_soma_ao_que_ja_estava_no_manifesto() {
        // Sem isto, embarcar duas vezes o mesmo produto duplicaria a linha em vez
        // de acumular.
        let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
        let product = product();
        let existing = cargo_of(10.0, 20.0);

        let change = ManifestTMImpl::new()
            .load(container.as_ref(), product.as_ref(), 5.0, Some(&existing))
            .expect("cabe");

        let cargo = change.cargo().expect("a linha continua existindo");
        assert_eq!(cargo.quantity(), 15.0);
        assert_eq!(cargo.weight(), 30.0);
        assert_eq!(change.container().current_weight(), 30.0);
    }

    #[test]
    fn a_carga_que_cabe_exatamente_e_aceita() {
        // O caso que a tolerância existe para proteger: a soma em ponto
        // flutuante pode passar da capacidade por uma fração invisível.
        let container = container_at(0.0, 20.0, ContainerStatus::Empty);
        let product = product();

        let change = ManifestTMImpl::new()
            .load(container.as_ref(), product.as_ref(), 10.0, None)
            .expect("20 kg em 20 kg de capacidade cabe");

        assert_eq!(change.container().current_weight(), 20.0);
    }

    #[test]
    fn recusa_carga_que_nao_cabe() {
        let container = container_at(0.0, 10.0, ContainerStatus::Empty);
        let product = product();

        let error = ManifestTMImpl::new()
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

            let error = ManifestTMImpl::new()
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
            let error = ManifestTMImpl::new()
                .load(container.as_ref(), product.as_ref(), bad, None)
                .err()
                .unwrap_or_else(|| panic!("quantidade {bad} deveria ser recusada"));

            assert!(matches!(error, ManifestError::InvalidQuantity));
        }
    }

    #[test]
    fn desembarcar_tudo_esvazia_o_conteiner_e_limpa_o_manifesto() {
        // Esvaziou: em vez de remover linha a linha, o manifesto inteiro vai
        // junto e o contêiner volta a `Empty`.
        let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
        let product = product();
        let existing = cargo_of(10.0, 20.0);

        let change = ManifestTMImpl::new()
            .unload(container.as_ref(), product.as_ref(), 10.0, Some(&existing))
            .expect("há 10 unidades embarcadas");

        assert_eq!(change.container().current_weight(), 0.0);
        assert_eq!(change.container().status(), ContainerStatus::Empty);
        assert!(change.clear_manifest());
        assert!(change.cargo().is_none());
        assert_eq!(change.event(), TelemetryEvent::Unload);
    }

    #[test]
    fn desembarcar_um_produto_por_completo_derruba_so_a_linha_dele() {
        // O contêiner segue com carga de outro produto, então só a linha deste
        // desaparece — e o manifesto NÃO é limpo.
        let container = container_at(50.0, 1000.0, ContainerStatus::Loading);
        let product = product();
        let existing = cargo_of(10.0, 20.0);

        let change = ManifestTMImpl::new()
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

        let change = ManifestTMImpl::new()
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

        let error = ManifestTMImpl::new()
            .unload(container.as_ref(), product.as_ref(), 11.0, Some(&existing))
            .err()
            .expect("não há 11 unidades");

        assert!(matches!(error, ManifestError::InsufficientCargo));
    }

    #[test]
    fn recusa_desembarcar_o_que_nunca_foi_embarcado() {
        let container = container_at(20.0, 1000.0, ContainerStatus::Loading);
        let product = product();

        let error = ManifestTMImpl::new()
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

            let error = ManifestTMImpl::new()
                .unload(container.as_ref(), product.as_ref(), 1.0, Some(&existing))
                .err()
                .unwrap_or_else(|| panic!("{status} não pode ser descarregado"));

            assert!(
                matches!(error, ManifestError::UnloadRequiresLoading),
                "status: {status}"
            );
        }
    }
}
