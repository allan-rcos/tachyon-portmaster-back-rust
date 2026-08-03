//! Hashing.
//!
//! Dois hashes com propósitos opostos, e por isso dois traits:
//!
//! * [`PasswordHasher`] precisa ser **lento e salgado** — é o que torna um
//!   vazamento do banco caro de explorar.
//! * [`IndexHasher`] precisa ser **rápido e determinístico** — é chave de
//!   lookup, e um salt por chamada impediria encontrar o que foi gravado.
//!
//! Trocá-los seria um desastre silencioso nas duas direções, então nunca são o
//! mesmo trait. Ambos são `pub(crate)`: hash de senha é regra do domínio, e
//! deixá-lo alcançável de fora permitiria gravar uma senha sem passar pelo
//! TableModule.

pub(crate) mod argon2;
pub(crate) mod xxhash;

/// Protege uma senha para armazenamento e confere uma tentativa.
pub(crate) trait PasswordHasher {
    /// Deriva o hash a ser guardado. A senha em claro nunca é persistida.
    fn hash(&self, plain: &str) -> String;

    /// Confere uma tentativa contra o hash guardado.
    fn verify(&self, plain: &str, hash: &str) -> bool;
}

/// Reduz um valor a uma chave curta e opaca de lookup.
///
/// O marcador guarda o digest, não o valor: a marca fica leve e o refresh token
/// original não pode ser reconstruído a partir do que está em memória.
pub(crate) trait IndexHasher {
    /// O digest do valor, estável entre chamadas e execuções.
    fn hash(&self, plain: &str) -> String;
}
