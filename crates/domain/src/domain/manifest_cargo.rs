//! O contrato de leitura de uma linha do manifesto.

use chrono::{DateTime, Utc};

/// Uma linha do manifesto: quanto de um produto está num contêiner.
///
/// É uma entidade **fraca** — satélite do contêiner, sem sentido sozinha. Por
/// isso carrega só `created_at`: não é atualizável nem sofre soft-delete, e
/// mudar uma linha é removê-la e recriá-la.
pub trait ManifestCargo: Send + Sync {
    /// Contêiner que carrega o item, em base62.
    fn container_id(&self) -> &str;

    /// Produto embarcado, em base62.
    fn product_id(&self) -> &str;

    /// Quantidade embarcada.
    fn quantity(&self) -> f64;

    /// Peso correspondente, em quilos.
    fn weight(&self) -> f64;

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;
}
