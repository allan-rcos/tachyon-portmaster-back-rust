//! Os DTOs de escrita: um por operação.
//!
//! Um Command é o vocabulário do `app` — o que a apresentação preenche e o caso
//! de uso consome. O `domain` não o conhece: o `TableModule` recebe valores
//! soltos, nunca um Command.

pub mod account;
pub mod container;
pub mod manifest;
pub mod marker;
pub mod product;
pub mod role;
pub mod session;
pub mod user;
