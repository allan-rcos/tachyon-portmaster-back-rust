//! Os parâmetros de querystring que as listagens aceitam.
//!
//! Cada listagem tem os seus, num arquivo só, porque o que elas aceitam diverge
//! — a de usuários pagina por página e as demais por cursor, e uni-las numa
//! struct só faria toda rota carregar campos que ela ignora.

pub mod container_page_params;
pub mod page_params;
pub mod search_params;
pub mod summary_page_params;
pub mod user_page_params;
