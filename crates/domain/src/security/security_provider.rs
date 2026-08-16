//! Quem serve os hashers.
//!
//! Nada é guardado aqui, e nada é configurável: os parâmetros dos dois são
//! constantes de build, porque mudá-los invalidaria o que já está gravado.

use crate::security::intern::argon2_hasher::argon2_hasher;
use crate::security::intern::xx_index_hasher::xx_index_hasher;
use crate::security::{IndexHasher, PasswordHasher};

/// Os hashers do domínio, um por propósito.
pub(crate) struct SecurityProvider;

impl SecurityProvider {
    /// O hasher de senha — lento e salgado, de propósito.
    pub(crate) fn password() -> impl PasswordHasher + Send + Sync + Clone + use<> + 'static {
        argon2_hasher()
    }

    /// O digest de indexação — rápido e determinístico, de propósito.
    pub(crate) fn index() -> impl IndexHasher + Send + Sync + Clone + use<> + 'static {
        xx_index_hasher()
    }
}
