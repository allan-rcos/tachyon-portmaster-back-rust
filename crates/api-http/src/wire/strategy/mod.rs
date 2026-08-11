//! As strategies de serialização, uma por formato.
//!
//! Cada uma conhece um formato e nada sobre o payload — é o que permite que a
//! mesma mensagem saia nos dois sem que nenhuma das duas saiba da outra.
//!
//! Quem escolhe entre elas é o contexto, e ele mora no middleware: o
//! `EncodeContext` na saída, o `DecodeContext` na entrada. Aqui ficam só as
//! strategies e os dois contratos que elas implementam, que é o que sobra do
//! Strategy pattern quando o contexto vira escopo de requisição.

pub(crate) mod decode_strategy;
pub(crate) mod encode_strategy;
pub(crate) mod flatbuffers_decode_strategy;
pub(crate) mod flatbuffers_encode_strategy;
pub(crate) mod json_decode_strategy;
pub(crate) mod json_encode_strategy;
