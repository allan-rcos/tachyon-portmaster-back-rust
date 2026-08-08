//! O cabeçalho HTTP do identificador de requisição.

use axum::http::HeaderName;

/// O cabeçalho que devolve o id ao cliente.
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
