//! Um contêiner de domínio, montado para o teste.

use chrono::{DateTime, TimeZone as _, Utc};
use portmaster_domain::domain::Container;
use portmaster_domain::enums::ContainerStatus;

/// Um contêiner que o teste controla.
#[derive(Debug, Clone)]
pub(crate) struct StubContainer {
    /// Identidade em base62.
    id: String,
    /// O status em que o teste o quer.
    status: ContainerStatus,
}

impl StubContainer {
    /// O contêiner deste id e status, dentro do `Box` do table module.
    pub(crate) fn boxed(id: &str, status: ContainerStatus) -> Box<dyn Container> {
        Box::new(Self {
            id: id.to_owned(),
            status,
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

impl Container for StubContainer {
    fn id(&self) -> &str {
        &self.id
    }

    fn code(&self) -> &'static str {
        "TESTU1234567"
    }

    fn current_weight(&self) -> f64 {
        0.0
    }

    fn max_capacity(&self) -> f64 {
        1_000.0
    }

    fn status(&self) -> ContainerStatus {
        self.status
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
