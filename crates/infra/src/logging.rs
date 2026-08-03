//! Logging.
//!
//! Mora na `infra` por ser a camada mais interna que ainda faz I/O — e escrever
//! log é I/O. O `app` reexporta estes traits para a apresentação, que é onde os
//! middlewares os usam.
//!
//! A saída é JSON estruturado, não texto. Um log que se lê bem no terminal se
//! agrega mal: correlacionar todas as linhas de uma requisição exige que
//! `request_id` seja um campo, não parte de uma frase.

use std::collections::BTreeMap;

/// Cria loggers nomeados.
///
/// O nome identifica a origem — `auth`, `http`, `container` — e vira campo em
/// toda linha que aquele logger emitir.
pub trait LoggerFactory: Clone + Send + Sync + 'static {
    /// Um logger para o componente indicado.
    fn create(&self, name: &str) -> Logger;
}

/// Um logger nomeado, com campos acumulados.
///
/// É barato de clonar e de estender: `with_field` devolve um logger novo em vez
/// de alterar o corrente, o que permite carimbar o `request_id` num escopo sem
/// vazá-lo para os demais.
#[derive(Debug, Clone)]
pub struct Logger {
    name: String,
    fields: BTreeMap<String, String>,
}

impl Logger {
    /// Monta um logger para um componente.
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            fields: BTreeMap::new(),
        }
    }

    /// Um logger igual a este, mais um campo.
    #[must_use]
    pub fn with_field(&self, key: &str, value: impl Into<String>) -> Self {
        let mut fields = self.fields.clone();
        fields.insert(key.to_owned(), value.into());

        Self {
            name: self.name.clone(),
            fields,
        }
    }

    /// Registra um evento de rotina.
    pub fn info(&self, message: &str) {
        tracing::info!(component = %self.name, fields = ?self.fields, "{message}");
    }

    /// Registra algo suspeito que não impediu a operação.
    pub fn warn(&self, message: &str) {
        tracing::warn!(component = %self.name, fields = ?self.fields, "{message}");
    }

    /// Registra uma falha.
    pub fn error(&self, message: &str) {
        tracing::error!(component = %self.name, fields = ?self.fields, "{message}");
    }

    /// O nome do componente.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A fábrica de loggers sobre `tracing`.
#[derive(Debug, Clone, Default)]
pub(crate) struct TracingLoggerFactory;

impl TracingLoggerFactory {
    /// Monta a fábrica.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl LoggerFactory for TracingLoggerFactory {
    fn create(&self, name: &str) -> Logger {
        Logger::new(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_logger_carrega_o_nome_do_componente() {
        let logger = TracingLoggerFactory::new().create("auth");
        assert_eq!(logger.name(), "auth");
    }

    #[test]
    fn acrescentar_campo_nao_altera_o_logger_de_origem() {
        // É o que permite carimbar o request_id num escopo sem que ele vaze para
        // as demais requisições que compartilham o mesmo logger base.
        let base = TracingLoggerFactory::new().create("http");
        let scoped = base.with_field("request_id", "abc123");

        assert!(base.fields.is_empty());
        assert_eq!(
            scoped.fields.get("request_id").map(String::as_str),
            Some("abc123")
        );
    }

    #[test]
    fn campos_se_acumulam() {
        let logger = TracingLoggerFactory::new()
            .create("http")
            .with_field("request_id", "abc")
            .with_field("user_id", "U1");

        assert_eq!(logger.fields.len(), 2);
    }
}
