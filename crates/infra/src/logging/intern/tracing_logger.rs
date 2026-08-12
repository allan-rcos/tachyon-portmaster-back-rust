//! O logger sobre o `tracing`. Não sai do crate.

use crate::logging::Logger;
use std::collections::BTreeMap;
use std::sync::RwLock;
use tracing::{Level, Span};
use tracing_subscriber::registry::LookupSpan as _;
use tracing_subscriber::Registry;

/// Os campos que uma tarefa acumulou, pendurados no span que a envolve.
///
/// Mora nas *extensions* do span, e não num campo do logger, porque é assim que
/// um campo posto no middleware alcança uma linha emitida três camadas abaixo:
/// quem emite não precisa ter o logger que o gravou, basta estar dentro do mesmo
/// span. Campo de macro não serviria — o conjunto deles é fixado na expansão, e
/// [`Span::record`] com chave de fora dele não grava nada.
///
/// `BTreeMap` e não `HashMap` pelo mesmo motivo de sempre: a ordem estável faz
/// duas linhas do mesmo evento saírem iguais, o que importa para quem faz diff
/// de log. `RwLock` porque a leitura empresta o span de forma compartilhada, e a
/// escrita vem de outra profundidade da mesma tarefa.
struct SpanFields(RwLock<BTreeMap<String, String>>);

/// Grava um campo no span corrente.
///
/// Sem span aberto — o boot, o `panic::set_hook`, um teste que não instalou
/// subscriber — não há onde gravar e a chamada não faz nada. Perder o campo é
/// melhor do que recusar a linha, e é a mesma escolha que o
/// [`SystemLogger`](crate::logging::SystemLogger) faz ao servir um logger antes
/// de alguém o ter instalado.
fn record_on_span(key: &str, value: String) {
    Span::current().with_subscriber(|(id, dispatch)| {
        let Some(registry) = dispatch.downcast_ref::<Registry>() else {
            return;
        };
        let Some(span) = registry.span(id) else {
            return;
        };

        let mut extensions = span.extensions_mut();

        if extensions.get_mut::<SpanFields>().is_none() {
            extensions.insert(SpanFields(RwLock::new(BTreeMap::new())));
        }

        if let Some(fields) = extensions.get_mut::<SpanFields>() {
            if let Ok(mut fields) = fields.0.write() {
                fields.insert(key.to_owned(), value);
            }
        }
    });
}

/// Acrescenta um par à linha, com o separador se já houver algo antes.
fn append(rendered: &mut String, key: &str, value: &str) {
    if !rendered.is_empty() {
        rendered.push_str(", ");
    }

    rendered.push_str(key);
    rendered.push('=');
    rendered.push_str(value);
}

/// Um logger nomeado que escreve no `tracing`.
///
/// Este é o único lugar do sistema que chama uma macro de **evento**. Todo o
/// resto recebe um [`Logger`] pelo construtor e fala com ele.
///
/// Não guarda campo nenhum: o que se acumula pertence à tarefa, e mora no span.
/// Dois loggers de componentes diferentes, criados em pontos diferentes, veem o
/// mesmo conjunto enquanto correrem sob o mesmo span.
#[derive(Debug, Clone)]
pub(crate) struct TracingLogger {
    /// O nome do logger, que vira o alvo da linha.
    name: String,
}

impl TracingLogger {
    /// Monta um logger para um componente.
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    /// Junta os campos do span aos do instante, na ordem em que saem na linha.
    ///
    /// A string é montada **antes** de a macro ser chamada, e não por um
    /// `Display` preguiçoso, porque o `tracing` bloqueia reentrância: enquanto um
    /// callback do subscriber roda, `can_enter` está desligado e
    /// [`Span::current`] devolve span nenhum. Adiar a leitura para a hora da
    /// formatação faria os campos do span sumirem da linha em silêncio.
    ///
    /// Quem chama já conferiu o nível, então a alocação só acontece em linha que
    /// vai mesmo ser emitida.
    fn render<const N: usize>(instant: [(&str, &str); N]) -> String {
        let mut rendered = String::new();

        Span::current().with_subscriber(|(id, dispatch)| {
            let Some(registry) = dispatch.downcast_ref::<Registry>() else {
                return;
            };
            let Some(span) = registry.span(id) else {
                return;
            };

            for span in span.scope().from_root() {
                let extensions = span.extensions();

                let Some(fields) = extensions.get::<SpanFields>() else {
                    continue;
                };
                let Ok(fields) = fields.0.read() else {
                    continue;
                };

                for (key, value) in fields.iter() {
                    append(&mut rendered, key, value);
                }
            }
        });

        for (key, value) in instant {
            append(&mut rendered, key, value);
        }

        rendered
    }
}

impl Logger for TracingLogger {
    fn with_field(&self, key: &str, value: impl Into<String>) {
        record_on_span(key, value.into());
    }

    fn info<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        if tracing::enabled!(Level::INFO) {
            tracing::info!(component = %self.name, fields = %Self::render(fields), "{message}");
        }
    }

    fn warn<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        if tracing::enabled!(Level::WARN) {
            tracing::warn!(component = %self.name, fields = %Self::render(fields), "{message}");
        }
    }

    fn error<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        if tracing::enabled!(Level::ERROR) {
            tracing::error!(component = %self.name, fields = %Self::render(fields), "{message}");
        }
    }

    fn debug<const N: usize>(&self, message: &str, fields: [(&str, &str); N]) {
        if tracing::enabled!(Level::DEBUG) {
            tracing::debug!(component = %self.name, fields = %Self::render(fields), "{message}");
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
#[path = "tests/tracing_logger_test.rs"]
mod tests;
