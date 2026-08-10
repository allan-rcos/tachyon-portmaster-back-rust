//! A declaração da memória na slice do escopo.
//!
//! O arquivo inteiro é a declaração — e ele não conhece o do banco, nem o
//! contrário. Nada no sistema afirma que os dois participam juntos; o linker é
//! quem junta.

use linkme::distributed_slice;

use crate::scope::memory::intern::moka_context::MokaContext;
use crate::scope::scope_layer::ScopeLayer;
use crate::scope::scope_layers::SCOPE_LAYERS;

/// A memória participa de todo escopo aberto neste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(SCOPE_LAYERS)]
static MEMORY: ScopeLayer = ScopeLayer {
    install: MokaContext::install,
};
