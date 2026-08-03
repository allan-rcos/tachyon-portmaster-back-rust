//! Tudo que vive em memória em vez do banco.
//!
//! Três coisas diferentes moram aqui, e vale distingui-las:
//!
//! * **Metadados de sistema** ([`metadata`]) — permissões e grupos, preenchidos
//!   no boot e imutáveis depois. Não têm TTL: um metadado despejado seria uma
//!   permissão sumindo do catálogo com o processo ainda de pé.
//! * **Marcadores** ([`marker`]) — booleanos com prazo. Têm TTL por entrada,
//!   porque cada sessão vence no seu próprio tempo.
//! * **Cache de leitura** ([`read`]) — resultados de consulta. Tem TTL curto e é
//!   invalidado por escrita.
//!
//! O que os três compartilham é a razão de não estarem no banco: são lidos com
//! frequência alta e não são a fonte da verdade de nada que precise sobreviver a
//! um restart.

pub(crate) mod marker;
pub(crate) mod metadata;
pub mod read;

pub use read::ReadCache;
