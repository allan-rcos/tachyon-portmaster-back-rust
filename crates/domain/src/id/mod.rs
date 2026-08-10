//! Os três sabores de gerador de id.
//!
//! Emitir identidade é contrato do domínio, e não da persistência: quem decide
//! quem uma entidade é são as suas regras, não o repositório que a grava. Por
//! isso os três moram aqui, como em `src/Domain/ID` do PHP — e não só o do
//! banco.
//!
//! | Trait | Para quê | Impl |
//! |---|---|---|
//! | `DatabaseIdGenerator` | chave primária, ordenada por tempo | Snowflake, em base62 |
//! | [`SequentialIdGenerator`] | ordena mas não vira coluna: `request_id` | xid |
//! | [`RandomIdGenerator`] | precisa ser impossível de adivinhar: refresh token | `NanoID` |
//!
//! O do banco é `pub(crate)`: um gerador de identidade nas mãos do `app`
//! permitiria montar uma entidade sem passar pelo `TableModule`, que é
//! justamente onde a validação mora. Os outros dois são públicos porque o id que
//! eles emitem não é identidade de entidade — ninguém consegue forjar uma linha
//! com um `request_id`.
//!
//! A **estratégia** do gerador de banco é escolhida por feature de compilação —
//! decisão de arquitetura, não um `if` de runtime. Os **parâmetros de
//! identidade** (`cluster_id`/`server_id`) são de deploy e chegam por segredo.

pub mod random_id_generator;
pub mod sequential_id_generator;

pub(crate) mod database_id_generator;
pub(crate) mod intern;

pub use random_id_generator::RandomIdGenerator;
pub use sequential_id_generator::SequentialIdGenerator;

pub(crate) use database_id_generator::DatabaseIdGenerator;
