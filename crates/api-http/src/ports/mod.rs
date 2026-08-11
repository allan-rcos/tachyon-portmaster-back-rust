//! As portas desta camada, e os adaptadores que as atendem.
//!
//! Vocabulário hexagonal, e ele é literal aqui: um `*_port.rs` (ou o contrato de
//! mesmo nome do diretório) declara o que a apresentação **precisa**, e o
//! `adapter/` ao lado é a única implementação — o resto do crate depende do
//! contrato e nunca nomeia a impl.
//!
//! `adapter` em vez de `intern`, que é o nome usado no resto do repositório, e a
//! diferença é de propósito: `intern` só diz "não sai do módulo", enquanto
//! `adapter` diz **o que** o arquivo é. O nome só vale onde existe uma porta do
//! outro lado, então `controllers/intern` e `middleware/intern` continuam
//! `intern`.

pub(crate) mod cookie;
pub(crate) mod error;
pub(crate) mod token;

pub(crate) mod session_policy;
