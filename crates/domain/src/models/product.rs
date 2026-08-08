//! O contrato de leitura de `Product`.

use chrono::{DateTime, Utc};

use crate::enums::RiskClass;

/// Um item catalogado, embarcável num contêiner.
pub trait Product: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Nome de exibição.
    fn name(&self) -> &str;

    /// Massa por unidade, em quilos.
    ///
    /// É o que converte uma quantidade embarcada em peso, e por isso não pode
    /// ser zero: um produto sem densidade encheria um contêiner sem nunca
    /// atingir a capacidade.
    fn density(&self) -> f64;

    /// Classe de risco, na numeração da ONU.
    fn risk_class(&self) -> RiskClass;

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
}
