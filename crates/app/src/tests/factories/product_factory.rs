//! Um produto de domínio, montado para o teste.

use chrono::{DateTime, TimeZone as _, Utc};
use portmaster_domain::domain::Product;
use portmaster_domain::enums::RiskClass;

/// Um produto que o teste controla.
#[derive(Debug, Clone)]
pub(crate) struct StubProduct {
    /// Identidade em base62.
    id: String,
    /// Nome comercial.
    name: String,
    /// Quilos por litro.
    density: f64,
}

impl StubProduct {
    /// O produto deste id, dentro do `Box` que o table module devolveria.
    pub(crate) fn boxed(id: &str, density: f64) -> Box<dyn Product> {
        Box::new(Self {
            id: id.to_owned(),
            name: "Soja".to_owned(),
            density,
        })
    }
}

/// O instante fixo das datas.
///
/// Fixo porque nenhuma asserção depende delas: o que se testa é orquestração, e
/// uma data variável só faria o teste parecer sensível ao relógio.
fn epoch() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("a data é válida e não é ambígua")
}

impl Product for StubProduct {
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
        RiskClass::None
    }

    fn created_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn updated_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        None
    }
}
