//! As implementações sobre MariaDB.
//!
//! Todas rodam na transação corrente, obtida do acessor interno da
//! [`UnitOfWork`](crate::database::UnitOfWork) — nenhuma abre transação por
//! conta própria. Quem delimita a operação de negócio é o `app`.
//!
//! ## Soft-delete
//!
//! Entidade forte não é apagada: `delete` grava `deleted_at`, e **toda** leitura
//! filtra `deleted_at IS NULL`. Esquecer esse filtro num `SELECT` faz linhas
//! removidas reaparecerem, então ele está em todas as consultas daqui — inclusive
//! nas de unicidade, porque um e-mail liberado por remoção precisa poder ser
//! reusado.

pub(crate) mod container;
pub(crate) mod manifest;
pub(crate) mod product;
pub(crate) mod role;
pub(crate) mod user;
