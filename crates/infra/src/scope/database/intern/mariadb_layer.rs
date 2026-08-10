//! A declaração do banco na slice do escopo.
//!
//! O arquivo inteiro é a declaração. Nenhum outro lugar do sistema menciona que
//! o banco participa do escopo — é o linker que junta, e é por isso que
//! acrescentar um contexto novo não toca em arquivo nenhum já existente.

use linkme::distributed_slice;

use crate::scope::database::intern::mariadb_context::MariaDbContext;
use crate::scope::scope_layer::ScopeLayer;
use crate::scope::scope_layers::SCOPE_LAYERS;

/// O banco participa de todo escopo aberto neste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(SCOPE_LAYERS)]
static MARIADB: ScopeLayer = ScopeLayer {
    install: MariaDbContext::install,
};
