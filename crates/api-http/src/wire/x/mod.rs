//! Os contratos que ligam um VO aos seus DTOs.
//!
//! Um VO por mensagem, dois DTOs por VO — um por formato — e uma trait que os
//! amarra. As strategies falam com estas traits e nunca com um VO concreto, e é
//! isso que faz acrescentar um formato ser trabalho local.

pub(crate) mod request_x;
pub(crate) mod response_x;
