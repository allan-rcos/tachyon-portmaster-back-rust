//! Como esta camada é montada.
//!
//! Um arquivo só: o provider da camada, que é a borda do crate. Os providers de
//! diretório — `scope/`, `repository/`, `query/`, `logging/` — ficam privados
//! atrás dele.

pub mod infra_provider;
