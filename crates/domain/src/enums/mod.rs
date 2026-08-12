//! Valores imutáveis que mudam a regra de negócio.
//!
//! Um valor que o domínio nunca altera e que decide o comportamento é um enum —
//! não uma linha de banco nem uma string solta. Fechá-lo no código é o ponto:
//! um `match` sobre o status obriga o compilador a apontar cada lugar que
//! esqueceu de tratar um estado novo, o que uma string jamais faria.
//!
//! A representação numérica de cada variante é o que vai ao banco e ao fio. Os
//! valores são fixos e **não podem ser reordenados** — mudar o índice de uma
//! variante reinterpreta silenciosamente as linhas já gravadas.

pub mod container_status;
pub mod risk_class;
pub mod telemetry_event;
pub mod unknown_index;

pub use container_status::ContainerStatus;
pub use risk_class::RiskClass;
pub use telemetry_event::TelemetryEvent;
pub use unknown_index::UnknownIndex;
