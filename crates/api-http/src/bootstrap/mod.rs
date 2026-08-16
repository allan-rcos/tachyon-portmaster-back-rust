//! Como esta camada é montada.
//!
//! Um arquivo só: o provider da camada. Ele é a fronteira interna da
//! apresentação — o `router` pede controller aqui e não conhece os providers de
//! `controllers/`, `middleware/` e `ports/token/`.
//!
//! Nada disto sobrevive ao boot: o que fica de pé é o router.

pub(crate) mod api_provider;
