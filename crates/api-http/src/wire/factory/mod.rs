//! As factories: sabem os dados, e nada sobre o formato.
//!
//! Esta é metade da separação que sustenta o wire. Uma factory conhece **o
//! quê** — quais campos a mensagem tem e de onde eles vêm — e é indiferente ao
//! **como**. A outra metade são as [strategies](super::strategy), que conhecem
//! um formato e nada sobre o payload.
//!
//! Manter as duas separadas é o que faz um terceiro formato custar uma strategy
//! nova, e não uma varredura por todas as mensagens.

pub(crate) mod renderable;
pub(crate) mod request_factory;
pub(crate) mod response_factory;
