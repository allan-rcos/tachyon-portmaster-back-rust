//! O que a sessão carrega no fio.
//!
//! Dois artefatos com naturezas opostas: o access token é auto-contido e
//! assinado, e o refresh é opaco e revogável. Ficam separados porque a única
//! coisa que compartilham é o nome.

pub mod refresh_token;
pub mod token_service;
