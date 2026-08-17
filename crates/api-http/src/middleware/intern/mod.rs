//! Os layers que abrem os escopos, e os contextos que os atendem.
//!
//! Um contexto é um par: o `tokio::task_local!` onde o estado da requisição vive
//! e o adaptador ZST que serve a porta correspondente lendo dele. O escritor é
//! `pub(super)` — só o layer irmão o alcança.
//!
//! Nem todo layer tem contexto. `recover` e `timeout` não guardam nada da
//! requisição; são middleware e só.
//!
//! E nem todo contexto mora aqui. `meta_event` e `cache_status` trabalham sobre
//! a pilha de eventos, que é do `app`: quem emite é um caso de uso, e um escopo
//! definido nesta camada estaria fora do alcance de quem escreve nele. O par
//! continua o mesmo — um layer abre, outro lê —, só que o `task_local!` está do
//! outro lado da fronteira e chega aqui como contrato.

pub(crate) mod cache_status_layer;
pub(crate) mod cookie_context;
pub(crate) mod cookie_layer;
pub(crate) mod decode_context;
pub(crate) mod decode_layer;
pub(crate) mod encode_context;
pub(crate) mod encode_layer;
pub(crate) mod logging_layer;
pub(crate) mod meta_event_layer;
pub(crate) mod recover_layer;
pub(crate) mod request_id_context;
pub(crate) mod request_id_layer;
pub(crate) mod session_context;
pub(crate) mod session_layer;
pub(crate) mod timeout_layer;
