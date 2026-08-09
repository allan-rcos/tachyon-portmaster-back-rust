//! Os middlewares transversais.
//!
//! Cada um é um par: o `tower::Service` que faz o trabalho e o `tower::Layer`
//! que o aplica. A pilha é composta por generics — cada `.layer()` embrulha o
//! serviço num tipo novo — então não há `dyn` nem `middleware::from_fn` solto em
//! ponto nenhum dela.

pub(crate) mod logging;
pub(crate) mod logging_layer;
pub(crate) mod recover;
pub(crate) mod recover_layer;
pub(crate) mod request_id;
pub(crate) mod request_id_header;
pub(crate) mod request_id_layer;
pub(crate) mod timeout;
pub(crate) mod timeout_layer;
pub(crate) mod token;
pub(crate) mod token_layer;
