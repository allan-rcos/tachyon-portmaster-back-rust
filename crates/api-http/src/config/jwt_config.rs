//! O que o token de sessão tem de segredo e de identidade.

use portmaster_app::SecretString;

/// O que o token de sessão tem de segredo e de identidade.
///
/// Só isto vem do ambiente. Validade, nomes de cookie, `Secure` e `SameSite`
/// eram seis variáveis a mais e viraram a `SessionPolicy`: não são segredo nem
/// identidade de deploy, são o que a API promete, e duas instâncias da mesma
/// versão não deveriam poder discordar sobre isso.
///
/// O padrão do `secret` é vazio de propósito — não existe padrão seguro para um
/// segredo de assinatura. Quem o exige é o `JwtChain`.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Segredo de assinatura HS256.
    pub secret: SecretString,
    /// Emissor, gravado e conferido na claim `iss`.
    pub issuer: String,
}

impl Default for JwtConfig {
    /// O emissor que o PHP usava, e nenhum segredo.
    fn default() -> Self {
        Self {
            secret: SecretString::from(String::new()),
            issuer: "tachyon/portmaster".to_owned(),
        }
    }
}
