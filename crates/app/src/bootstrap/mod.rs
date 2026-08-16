//! Como esta camada é montada.
//!
//! Um arquivo só: o provider da camada, que é a borda do crate e também o
//! ponto de partida do processo — o `boot` mora nele. O `ServicesProvider` fica
//! privado atrás dele.

pub mod app_provider;
