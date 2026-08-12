//! Uma linha do manifesto, montada para o teste.

use chrono::{DateTime, TimeZone as _, Utc};
use portmaster_domain::domain::ManifestCargo;

/// Uma linha que o teste controla.
pub(crate) struct StubCargo {
    /// O contêiner.
    container_id: String,
    /// O produto.
    product_id: String,
    /// Quantas unidades.
    quantity: f64,
}

impl StubCargo {
    /// A linha destes ids, dentro do `Box` que o domínio devolveria.
    pub(crate) fn boxed(
        container_id: &str,
        product_id: &str,
        quantity: f64,
    ) -> Box<dyn ManifestCargo> {
        Box::new(Self {
            container_id: container_id.to_owned(),
            product_id: product_id.to_owned(),
            quantity,
        })
    }
}

impl ManifestCargo for StubCargo {
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
        self.quantity * 0.75
    }

    fn created_at(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .expect("a data é válida e não é ambígua")
    }
}
