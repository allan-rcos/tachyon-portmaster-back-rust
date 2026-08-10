//! O logger sobre o `tracing`. Não sai do crate.

use crate::logging::Logger;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Os campos de uma linha: os do logger e os do instante, lado a lado.
///
/// Existe para que os dois conjuntos saiam num campo só sem serem copiados para
/// um mapa novo a cada evento. Ele só empresta os dois e os escreve em ordem na
/// hora em que o subscriber formata — se o nível estiver desligado, nem isso.
struct Fields<'a, const N: usize> {
    /// Os campos fixos do logger.
    inherited: &'a BTreeMap<String, String>,
    /// Os campos que valem só para esta linha.
    instant: [(&'a str, &'a str); N],
}

impl<const N: usize> Display for Fields<'_, N> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let inherited = self
            .inherited
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()));

        for (position, (key, value)) in inherited.chain(self.instant).enumerate() {
            if position > 0 {
                formatter.write_str(", ")?;
            }

            write!(formatter, "{key}={value}")?;
        }

        Ok(())
    }
}

/// Um logger nomeado que escreve no `tracing`.
///
/// Este é o único lugar do sistema que chama uma macro de **evento**. Todo o
/// resto recebe um [`Logger`] pelo construtor e fala com ele.
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

    /// Junta os campos fixos aos do instante para a linha que vai sair.
    const fn render<'a, const N: usize>(
        &'a self,
        instant: [(&'a str, &'a str); N],
    ) -> Fields<'a, N> {
        Fields {
            inherited: &self.fields,
            instant,
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

    fn info<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        tracing::info!(component = %self.name, fields = %self.render(fields), "{message}");
    }

    fn warn<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        tracing::warn!(component = %self.name, fields = %self.render(fields), "{message}");
    }

    fn error<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        tracing::error!(component = %self.name, fields = %self.render(fields), "{message}");
    }

    fn debug<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        tracing::debug!(component = %self.name, fields = %self.render(fields), "{message}");
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::intern::tracing_logger_factory::TracingLoggerFactory;
    use crate::logging::LoggerFactory;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_logger_carrega_o_nome_do_componente() {
        let logger = TracingLoggerFactory::new().create("auth");
        assert_eq!(logger.name(), "auth");
    }

    /// Um campo de identidade não pode vazar para quem compartilha o logger de
    /// origem — é o que justifica devolver um logger novo em vez de mutar.
    #[test]
    fn acrescentar_campo_nao_altera_o_logger_de_origem() {
        let base = TracingLogger::new("http");
        let derived = base.with_field("instance", "worker-3");

        assert!(base.fields.is_empty());
        assert_eq!(
            derived.fields.get("instance").map(String::as_str),
            Some("worker-3")
        );
    }

    #[test]
    fn campos_se_acumulam() {
        let logger = TracingLogger::new("http")
            .with_field("instance", "worker-3")
            .with_field("region", "sa-east-1");

        assert_eq!(logger.fields.len(), 2);
    }

    #[test]
    fn a_linha_leva_os_campos_do_logger_e_os_do_instante() {
        let logger = TracingLogger::new("http").with_field("instance", "worker-3");

        assert_eq!(
            logger
                .render([("status", "200"), ("duration_ms", "7")])
                .to_string(),
            "instance=worker-3, status=200, duration_ms=7"
        );
    }

    /// Sem campo nenhum a linha não ganha separador solto.
    #[test]
    fn sem_campos_a_renderizacao_e_vazia() {
        assert_eq!(TracingLogger::new("http").render([]).to_string(), "");
    }
}
