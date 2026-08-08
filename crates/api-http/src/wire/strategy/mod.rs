//! As strategies: sabem um formato, e nada sobre o payload.
//!
//! Toda tabela de `FlatBuffers` se serializa igual, e toda struct serde também —
//! é por isso que uma strategy pode ser indiferente ao que está passando por
//! ela. A outra metade são as [factories](super::factory), que conhecem os dados
//! e nada sobre o formato.
//!
//! ## Os padrões da negociação não são simétricos
//!
//! Sem `Content-Type`, o corpo é lido como **`FlatBuffers`**; sem `Accept`, a
//! resposta sai em **JSON**. A assimetria herdada do PHP, e sensata: quem manda
//! corpo sem anunciar o tipo é um cliente nosso falando o formato nativo, e quem
//! não pede formato nenhum costuma ser um humano com um `curl`.

pub(crate) mod decode_strategy;
pub(crate) mod encode_strategy;
pub(crate) mod flatbuffers_decode_strategy;
pub(crate) mod flatbuffers_encode_strategy;
pub(crate) mod json_decode_strategy;
pub(crate) mod json_encode_strategy;
