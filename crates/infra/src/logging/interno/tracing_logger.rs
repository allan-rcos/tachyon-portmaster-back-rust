//! O logger sobre o `tracing`. Não sai do crate.

use crate::logging::Logger;
use std::collections::BTreeMap;

/// Um logger nomeado que escreve no `tracing`.
///
/// Este é o único lugar do sistema que chama uma macro de log. Todo o resto
/// recebe um [`Logger`] pelo construtor e fala com ele.
#[derive(Debug, Clone)]
pub(crate) struct TracingLogger {
    /// O nome do logger, que vira o alvo da linha.
    name: String,
    /// Os campos fixos deste logger, herdados por toda linha que ele emite.
    ///
    /// `BTreeMap` e não `HashMap`: a ordem estável faz duas linhas do mesmo
    /// evento saírem iguais, o que importa para quem faz diff de log.
    fields: BTreeMap<String, String>,
}

impl TracingLogger {
    /// Monta um logger para um componente.
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            fields: BTreeMap::new(),
        }
    }
}

impl Logger for TracingLogger {
    fn with_field(&self, key: &str, value: impl Into<String>) -> Self {
        let mut fields = self.fields.clone();
        fields.insert(key.to_owned(), value.into());

        Self {
            name: self.name.clone(),
            fields,
        }
    }

    fn info(&self, message: &str) {
        tracing::info!(component = %self.name, fields = ?self.fields, "{message}");
    }

    fn warn(&self, message: &str) {
        tracing::warn!(component = %self.name, fields = ?self.fields, "{message}");
    }

    fn error(&self, message: &str) {
        tracing::error!(component = %self.name, fields = ?self.fields, "{message}");
    }

    fn debug(&self, message: &str) {
        tracing::debug!(component = %self.name, fields = ?self.fields, "{message}");
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::interno::tracing_logger_factory::TracingLoggerFactory;
    use crate::logging::LoggerFactory;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_logger_carrega_o_nome_do_componente() {
        let logger = TracingLoggerFactory::new().create("auth");
        assert_eq!(logger.name(), "auth");
    }

    /// É o que permite carimbar o `request_id` num escopo sem que ele vaze para
    /// as demais requisições que compartilham o mesmo logger base.
    #[test]
    fn acrescentar_campo_nao_altera_o_logger_de_origem() {
        let base = TracingLogger::new("http");
        let scoped = base.with_field("request_id", "abc123");

        assert!(base.fields.is_empty());
        assert_eq!(
            scoped.fields.get("request_id").map(String::as_str),
            Some("abc123")
        );
    }

    #[test]
    fn campos_se_acumulam() {
        let logger = TracingLogger::new("http")
            .with_field("request_id", "abc")
            .with_field("user_id", "U1");

        assert_eq!(logger.fields.len(), 2);
    }
}
