//! Como a conexão com o banco negocia TLS.

/// Como a conexão com o banco trata TLS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatabaseSslMode {
    /// Sem TLS.
    ///
    /// A resposta certa para um banco em `127.0.0.1` ou numa subnet privada, e a
    /// errada para qualquer banco gerenciado — que recusa conexão em claro.
    #[default]
    Disabled,

    /// Criptografa sem validar o certificado.
    ///
    /// Resolve escuta passiva, não ataque ativo: quem consegue se pôr no meio
    /// apresenta o certificado que quiser.
    Required,

    /// Criptografa e valida a cadeia contra a CA configurada.
    VerifyCa,
}
