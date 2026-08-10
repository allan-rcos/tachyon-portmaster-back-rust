//! Hashing.
//!
//! Dois hashes com propósitos opostos, e por isso dois traits:
//!
//! * [`password_hasher::PasswordHasher`] precisa ser **lento e
//!   salgado** — é o que torna um vazamento do banco caro de explorar.
//! * [`index_hasher::IndexHasher`] precisa ser **rápido e
//!   determinístico** — é chave de lookup, e um salt por chamada impediria
//!   encontrar o que foi gravado.
//!
//! Trocá-los seria um desastre silencioso nas duas direções, então nunca são o
//! mesmo trait. Ambos são `pub(crate)`: hash de senha é regra do domínio, e
//! deixá-lo alcançável de fora permitiria gravar uma senha sem passar pelo
//! `TableModule`.

pub mod index_hasher;
pub mod password_hasher;

pub(crate) mod intern;

pub use index_hasher::IndexHasher;
pub use password_hasher::PasswordHasher;
