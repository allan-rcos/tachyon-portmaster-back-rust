//! Sob que nome cada cookie de sessão viaja.

/// Os dois cookies que carregam uma sessão.
///
/// Enum e não string porque o nome é a chave nos dois sentidos — quem escreve e
/// quem lê têm que concordar —, e uma string solta em dois arquivos é a forma
/// mais fácil de eles deixarem de concordar sem ninguém perceber.
///
/// Os valores são contrato com quem já roda o sistema: um cookie renomeado
/// desloga toda sessão aberta no momento do deploy, e a suíte de integração os
/// afirma por nome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CookieName {
    /// O access token assinado.
    Access,
    /// O refresh token opaco.
    Refresh,
}

impl CookieName {
    /// O nome como ele aparece no cabeçalho.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Access => "auth_token",
            Self::Refresh => "refresh_token",
        }
    }
}
