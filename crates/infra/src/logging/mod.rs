//! O log estruturado.
//!
//! Um [`Logger`] carrega campos e os repete em toda linha que escreve — é o que
//! faz `request_id` aparecer em cada evento de uma requisição sem que cada
//! chamador o passe de novo.
//!
//! A trait mora aqui e a impl mora em `interno`: este é o único crate que
//! conhece o `tracing`, e quem está acima recebe um `impl Logger` pela
//! [`LoggerFactory`]. Para os pontos sem construtor onde injetar — pânico,
//! função associada — há o [`SystemLogger`].

pub mod logger;
pub mod logger_factory;
pub mod system_logger;

pub(crate) mod interno;

pub use logger::Logger;
pub use logger_factory::LoggerFactory;
pub use system_logger::SystemLogger;
