//! O contrato de leitura de `Container`.

use chrono::{DateTime, Utc};

use crate::enums::ContainerStatus;

/// Um contêiner do pátio.
pub trait Container: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Identificador usado no pátio, único.
    fn code(&self) -> &str;

    /// Peso embarcado no momento, em quilos.
    ///
    /// Mantido junto com a escrita do item, na mesma transação — recalcular a
    /// soma do manifesto a cada consulta custaria uma agregação por leitura.
    fn current_weight(&self) -> f64;

    /// Capacidade máxima, em quilos.
    fn max_capacity(&self) -> f64;

    /// Onde está no ciclo de vida.
    fn status(&self) -> ContainerStatus;

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
}
