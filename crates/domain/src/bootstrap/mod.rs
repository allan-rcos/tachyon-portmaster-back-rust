//! Como esta camada é montada.
//!
//! Um arquivo só: o provider da camada, que é a borda do crate. Os providers de
//! diretório — `id/`, `security/`, `table_modules/` — ficam privados atrás dele.

pub mod domain_provider;
