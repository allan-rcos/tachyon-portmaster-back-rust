//! A implementação de domínio de uma linha do manifesto.

use chrono::{DateTime, Utc};

use crate::domain::ManifestCargo;

/// A implementação do domínio de [`ManifestCargo`].
pub struct ManifestCargoModel {
    /// O contêiner que carrega, em base62.
    container_id: String,
    /// O produto carregado, em base62.
    product_id: String,
    /// Quantas unidades.
    quantity: f64,
    /// O peso que essa quantidade representa, já pela densidade do produto.
    weight: f64,
    /// Quando a linha entrou no manifesto, em UTC.
    created_at: DateTime<Utc>,
}

impl ManifestCargoModel {
    /// Monta uma linha de manifesto.
    pub(crate) fn new(
        container_id: String,
        product_id: String,
        quantity: f64,
        weight: f64,
    ) -> Self {
        Self {
            container_id,
            product_id,
            quantity,
            weight,
            created_at: Utc::now(),
        }
    }
}

impl ManifestCargo for ManifestCargoModel {
    fn container_id(&self) -> &str {
        &self.container_id
    }

    fn product_id(&self) -> &str {
        &self.product_id
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
