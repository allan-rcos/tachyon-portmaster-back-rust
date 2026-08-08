//! A implementação de domínio de `Product`.

use chrono::{DateTime, Utc};

use crate::enums::RiskClass;
use crate::models::Product;

/// A implementação do domínio de [`Product`].
pub struct ProductModel {
    id: String,
    name: String,
    density: f64,
    risk_class: RiskClass,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ProductModel {
    /// Monta um produto a partir de campos já validados.
    pub(crate) fn new(id: String, name: String, density: f64, risk_class: RiskClass) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            density,
            risk_class,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Recria o model a partir de qualquer [`Product`].
    pub(crate) fn from_domain(source: &dyn Product) -> Self {
        Self {
            id: source.id().to_owned(),
            name: source.name().to_owned(),
            density: source.density(),
            risk_class: source.risk_class(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        }
    }

    /// Substitui os dados do catálogo, marcando a alteração.
    pub(crate) fn set_details(&mut self, name: String, density: f64, risk_class: RiskClass) {
        self.name = name;
        self.density = density;
        self.risk_class = risk_class;
        self.updated_at = Utc::now();
    }
}

impl Product for ProductModel {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn density(&self) -> f64 {
        self.density
    }

    fn risk_class(&self) -> RiskClass {
        self.risk_class
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}
