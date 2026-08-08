//! O log estruturado.
//!
//! Um [`Logger`] carrega campos e os repete em toda linha que escreve — é o que
//! faz `request_id` aparecer em cada evento de uma requisição sem que cada
//! chamador o passe de novo.

pub mod logger;
pub mod logger_factory;

pub(crate) mod interno;

pub use logger::Logger;
pub use logger_factory::LoggerFactory;
