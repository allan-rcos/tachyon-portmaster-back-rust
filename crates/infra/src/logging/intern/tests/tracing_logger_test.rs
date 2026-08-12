//! Os testes de `tracing_logger`.

use super::*;
use crate::logging::intern::tracing_logger_factory::TracingLoggerFactory;
use crate::logging::LoggerFactory;
use pretty_assertions::assert_eq;
use std::io::{Error as IoError, Result as IoResult, Write};
use std::sync::{Arc, Mutex};
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::layer::SubscriberExt as _;

/// As linhas que o subscriber escreveu, para o teste conferir.
///
/// Clonar compartilha o mesmo buffer: uma cópia vai para o formatador e a
/// outra fica com o teste.
#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Buffer {
    /// O que foi escrito até agora.
    fn contents(&self) -> String {
        let bytes = self.0.lock().expect("o buffer não está envenenado").clone();

        String::from_utf8(bytes).expect("o formatador escreve UTF-8")
    }
}

impl Write for Buffer {
    fn write(&mut self, buffer: &[u8]) -> IoResult<usize> {
        let mut sink = self
            .0
            .lock()
            .map_err(|_| IoError::other("o buffer está envenenado"))?;

        sink.extend_from_slice(buffer);

        Ok(buffer.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

/// Instala um subscriber com `Registry` por baixo, como o do processo.
///
/// É `Layered` e não um `Registry` pelado de propósito: em produção o
/// `fmt().json()` empilha uma camada em cima, e é o `downcast_ref` atravessar
/// essa pilha que faz os campos chegarem ao span.
fn subscriber() -> DefaultGuard {
    let subscriber = Registry::default().with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_default(subscriber)
}

#[test]
fn o_logger_carrega_o_nome_do_componente() {
    let logger = TracingLoggerFactory::new().create("auth");
    assert_eq!(logger.name(), "auth");
}

/// O que justifica gravar no span em vez de num campo do logger: quem emite
/// a linha não é quem gravou o campo, e nem tem o logger que gravou.
#[test]
fn um_campo_gravado_por_outro_logger_sai_na_linha() {
    let _guard = subscriber();
    let span = tracing::info_span!("request");
    let _entered = span.enter();

    TracingLogger::new("http").with_field("instance", "worker-3");

    assert_eq!(
        TracingLogger::render([("status", "200")]),
        "instance=worker-3, status=200"
    );
}

#[test]
fn campos_se_acumulam() {
    let _guard = subscriber();
    let span = tracing::info_span!("request");
    let _entered = span.enter();

    let logger = TracingLogger::new("http");
    logger.with_field("instance", "worker-3");
    logger.with_field("region", "sa-east-1");

    assert_eq!(
        TracingLogger::render([]),
        "instance=worker-3, region=sa-east-1"
    );
}

/// O campo do span de fora alcança quem corre num span de dentro.
#[test]
fn um_span_filho_enxerga_o_campo_do_pai() {
    let _guard = subscriber();
    let outer = tracing::info_span!("request");
    let _outer = outer.enter();

    let logger = TracingLogger::new("http");
    logger.with_field("request_id", "abc123");

    let inner = tracing::info_span!("query");
    let _inner = inner.enter();
    logger.with_field("table", "users");

    assert_eq!(TracingLogger::render([]), "request_id=abc123, table=users");
}

/// O que um span acumula não escapa para o irmão que corre fora dele.
#[test]
fn spans_diferentes_nao_se_misturam() {
    let _guard = subscriber();
    let logger = TracingLogger::new("http");

    tracing::info_span!("primeira").in_scope(|| {
        logger.with_field("request_id", "um");
    });

    tracing::info_span!("segunda").in_scope(|| {
        assert_eq!(TracingLogger::render([]), "");
    });
}

/// Sem span não há onde gravar, e isso não pode derrubar quem loga.
#[test]
fn sem_span_o_campo_e_descartado_em_silencio() {
    let _guard = subscriber();

    TracingLogger::new("http").with_field("instance", "worker-3");

    assert_eq!(TracingLogger::render([]), "");
}

/// Sem campo nenhum a linha não ganha separador solto.
#[test]
fn sem_campos_a_renderizacao_e_vazia() {
    assert_eq!(TracingLogger::render([]), "");
}

/// O caminho de produção inteiro, e não só o `render`.
///
/// Vale o teste separado porque é o `fmt().json()` do `main.rs` que decide o
/// que chega ao coletor: se o campo do span parasse no meio do caminho, os
/// testes acima continuariam verdes e a linha sairia sem ele.
#[test]
fn a_linha_em_json_carrega_o_campo_do_span() {
    let buffer = Buffer::default();
    let sink = buffer.clone();
    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(move || sink.clone());

    let guard = tracing::subscriber::set_default(Registry::default().with(layer));

    tracing::info_span!("request").in_scope(|| {
        let logger = TracingLogger::new("http");
        logger.with_field("request_id", "abc123");
        logger.info("requisição atendida", [("status", "200")]);
    });

    drop(guard);

    let line = buffer.contents();

    assert!(
        line.contains(r#""fields":"request_id=abc123, status=200""#),
        "a linha não trouxe o campo do span: {line}"
    );
}
