//! O token de sessão — assunto exclusivo desta camada.
//!
//! O access token e o refresh são coisas diferentes: um é assinado e carrega o
//! principal, o outro é opaco e só serve para pedir um access novo. Cada um tem
//! o seu arquivo.

pub(crate) mod refresh_token;
pub(crate) mod token_service;

pub(crate) mod adapter;
