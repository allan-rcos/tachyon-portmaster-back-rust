//! O contêiner: a unidade que se carrega, sela e despacha.

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

/// A implementação do domínio de [`Container`].
pub(crate) struct ContainerModel {
    id: String,
    code: String,
    current_weight: f64,
    max_capacity: f64,
    status: ContainerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ContainerModel {
    /// Monta um contêiner a partir de campos já validados.
    pub(crate) fn new(
        id: String,
        code: String,
        current_weight: f64,
        max_capacity: f64,
        status: ContainerStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            code,
            current_weight,
            max_capacity,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Recria o model a partir de qualquer [`Container`].
    pub(crate) fn from_domain(source: &dyn Container) -> Self {
        Self {
            id: source.id().to_owned(),
            code: source.code().to_owned(),
            current_weight: source.current_weight(),
            max_capacity: source.max_capacity(),
            status: source.status(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        }
    }

    /// Substitui a capacidade, marcando a alteração.
    pub(crate) fn set_max_capacity(&mut self, max_capacity: f64) {
        self.max_capacity = max_capacity;
        self.updated_at = Utc::now();
    }

    /// Substitui peso e status juntos, marcando a alteração.
    ///
    /// Os dois andam sempre juntos — carregar muda o peso e pode mudar o status,
    /// descarregar até o vazio faz as duas coisas — e separá-los abriria espaço
    /// para um contêiner com peso zero marcado como `Loading`.
    pub(crate) fn set_weight_and_status(&mut self, weight: f64, status: ContainerStatus) {
        self.current_weight = weight;
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Container for ContainerModel {
    fn id(&self) -> &str {
        &self.id
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn current_weight(&self) -> f64 {
        self.current_weight
    }

    fn max_capacity(&self) -> f64 {
        self.max_capacity
    }

    fn status(&self) -> ContainerStatus {
        self.status
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
