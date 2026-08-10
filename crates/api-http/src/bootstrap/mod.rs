//! Como esta camada é montada.
//!
//! O contrato dos factories, a implementação que os atende e a função que
//! destrincha a configuração e devolve os controllers prontos. Nada disto
//! sobrevive ao boot: o que fica de pé é o router.

pub(crate) mod api_provider;
pub(crate) mod provider;
pub(crate) mod register;
