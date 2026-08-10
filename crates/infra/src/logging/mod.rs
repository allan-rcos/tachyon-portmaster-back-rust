//! O log estruturado.
//!
//! Um [`Logger`] tem nome, e cada linha leva junto os campos daquele instante.
//!
//! A trait mora aqui e a impl mora em `intern`: este é o único crate que **emite
//! evento**, e quem está acima recebe um `impl Logger` pela [`LoggerFactory`].
//! Para os pontos sem construtor onde injetar — pânico, função associada — há o
//! [`SystemLogger`].
//!
//! ## O que correlaciona as linhas de uma requisição não está aqui
//!
//! É o span do `tracing`, aberto por quem sabe o que é uma requisição: o
//! middleware do transporte. Toda linha que sair enquanto ele estiver aberto —
//! do `app`, da `infra`, de qualquer profundidade — sai com o `request_id`
//! dentro, sem que nada disso precise recebê-lo por parâmetro. Era o que um
//! logger carimbado não conseguia fazer: ele só alcançava quem o tivesse em mãos,
//! e o `request_id` morria no middleware que o criou.

pub mod logger;
pub mod logger_factory;
pub mod system_logger;

pub(crate) mod intern;

pub use logger::Logger;
pub use logger_factory::LoggerFactory;
pub use system_logger::SystemLogger;
