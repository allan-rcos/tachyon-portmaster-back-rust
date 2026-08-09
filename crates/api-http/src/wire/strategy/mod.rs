//! As strategies de serialização, uma por formato.
//!
//! Cada uma conhece um formato e nada sobre o payload — é o que permite que a
//! mesma mensagem saia nos dois sem que nenhuma das duas saiba da outra. Quem
//! escolhe entre elas é o contexto: [`Encoder`](crate::wire::encoder::Encoder)
//! na saída, [`Decoder`](crate::wire::decoder::Decoder) na entrada.

pub(crate) mod decode_strategy;
pub(crate) mod encode_strategy;
pub(crate) mod flatbuffers_decode_strategy;
pub(crate) mod flatbuffers_encode_strategy;
pub(crate) mod json_decode_strategy;
pub(crate) mod json_encode_strategy;
