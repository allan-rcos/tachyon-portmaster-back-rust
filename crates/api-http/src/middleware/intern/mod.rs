//! Os layers que abrem os escopos, e os contextos que os atendem.
//!
//! Um contexto é um par: o `tokio::task_local!` onde o estado da requisição vive
//! e o adaptador ZST que serve a porta correspondente lendo dele. O escritor é
//! `pub(super)` — só o layer irmão o alcança.
//!
//! Nem todo layer tem contexto. `recover` e `timeout` não guardam nada da
//! requisição; são middleware e só.

pub(crate) mod logging_layer;
pub(crate) mod recover_layer;
pub(crate) mod request_id_context;
pub(crate) mod request_id_layer;
pub(crate) mod session_layer;
pub(crate) mod timeout_layer;
