//! A implementação das regras de manifesto.

use crate::domain::{Container, ManifestCargo, ManifestChange, Product};
use crate::enums::{ContainerStatus, TelemetryEvent};
use crate::error::ManifestError;
use crate::table_modules::intern::models::container_model::ContainerModel;
use crate::table_modules::intern::models::manifest_cargo_model::ManifestCargoModel;
use crate::table_modules::intern::models::manifest_change_model::ManifestChangeModel;
use crate::table_modules::ManifestTM;

/// A implementação. Não precisa de helper nenhum: é aritmética e regra pura.
#[derive(Clone)]
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

    /// Desembarca uma quantidade, e devolve o efeito no contêiner.
    ///
    /// Sem linha de manifesto, ou com menos do que se pediu, não há o que
    /// tirar: os dois casos saem como `InsufficientCargo`. A tolerância de
    /// `EPSILON` na comparação deixa passar o caso de descarregar exatamente
    /// tudo que está lá, que em ponto flutuante pode não bater na igualdade.
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

        // Produto zerado com o contêiner ainda carregado: só a linha dele cai.
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
#[path = "tests/manifest_tm_impl_test.rs"]
mod tests;
