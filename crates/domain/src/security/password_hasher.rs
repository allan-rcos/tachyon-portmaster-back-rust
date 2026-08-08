//! O contrato do hash que protege uma senha.

/// Protege uma senha para armazenamento e confere uma tentativa.
pub trait PasswordHasher {
    /// Deriva o hash a ser guardado. A senha em claro nunca é persistida.
    fn hash(&self, plain: &str) -> String;

    /// Confere uma tentativa contra o hash guardado.
    fn verify(&self, plain: &str, hash: &str) -> bool;
}
