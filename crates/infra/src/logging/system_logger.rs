//! O logger de sistema, alcançável de onde não há provider.

use crate::logging::interno::tracing_logger::TracingLogger;
use crate::logging::Logger;
use std::sync::OnceLock;

/// O nome que identifica as linhas emitidas sem um componente próprio.
const NAME: &str = "system";

/// A instância única, instalada no boot.
static INSTANCE: OnceLock<TracingLogger> = OnceLock::new();

/// O logger global do processo.
///
/// Existe para os pontos em que **não há** um construtor onde injetar um
/// logger: o `panic::set_hook`, o middleware que recolhe um pânico, funções
/// associadas. Fora daí, a regra continua sendo a de sempre — quem tem
/// construtor recebe o [`Logger`] pela
/// [`LoggerFactory`](crate::logging::LoggerFactory).
///
/// Ele não faz o `tracing` aparecer nas camadas de cima: continua sendo a
/// `infra` que fala com o `tracing`, e o que sai daqui é a mesma abstração.
pub struct SystemLogger;

impl SystemLogger {
    /// Fixa a instância do processo.
    ///
    /// Chamar duas vezes não é erro — a segunda é ignorada, porque um boot que
    /// falha ao instalar o logger não é motivo para derrubar o processo.
    pub fn install() {
        let _ = INSTANCE.set(TracingLogger::new(NAME));
    }

    /// O logger global.
    ///
    /// Se ninguém instalou ainda, devolve um equivalente em vez de recusar. O
    /// momento em que isto acontece — antes de o boot terminar — é justamente
    /// quando uma linha de log tem mais valor, e perdê-la para uma questão de
    /// ordem de inicialização seria o pior desfecho possível.
    pub fn get() -> impl Logger {
        INSTANCE.get_or_init(|| TracingLogger::new(NAME)).clone()
    }
}
