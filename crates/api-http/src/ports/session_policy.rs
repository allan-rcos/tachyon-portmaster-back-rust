//! Quanto a sessão vale, e sob que política os cookies dela viajam.

use std::time::Duration;

use cookie::SameSite;

/// A política de sessão, decidida em tempo de compilação.
///
/// Nada disto é segredo nem identidade de deploy — é **arquitetura**, e é por
/// isso que saiu do ambiente. Um `if` em produção sobre quanto tempo uma sessão
/// dura é um bug esperando o dia de errar, e o que se ganhava em troca era a
/// possibilidade de dois deploys da mesma versão discordarem sobre o que a API
/// promete.
///
/// ## Uma duração só para o token e para o cookie
///
/// [`Self::ACCESS_TTL`] é ao mesmo tempo o `exp` da claim e o `Max-Age` do
/// cookie que a carrega, e isso não é economia: eram duas variáveis de ambiente
/// (`APP_JWT_TTL` e o max-age do cookie) que **podiam divergir**, e divergindo
/// produziam um cookie que o navegador guarda depois de o token dentro dele já
/// não valer — ou o contrário, um token bom que o navegador jogou fora.
///
/// ## `Secure` segue o perfil de compilação
///
/// Release liga; debug não. O ambiente de desenvolvimento roda em HTTP puro, e
/// um cookie `Secure` simplesmente não seria enviado — a sessão nunca
/// funcionaria localmente. A imagem de contêiner é compilada em release, então
/// um build local que precise servir HTTP puro passa
/// `RUSTFLAGS="-C debug-assertions=on"`, que é o que o `Dockerfile` faz sob o
/// `ARG RUST_DEBUG_ASSERTIONS` e o que a suíte de integração usa.
///
/// Struct-namespace e não um `mod` de consts soltas: o módulo já é o arquivo, e
/// agrupá-las num tipo é o que mantém um export só por arquivo.
pub(crate) struct SessionPolicy;

impl SessionPolicy {
    /// Quanto vale um access token — e o cookie que o carrega.
    pub(crate) const ACCESS_TTL: Duration = Duration::from_secs(3600);

    /// Quanto vale um refresh token — e o cookie que o carrega.
    ///
    /// Quatorze dias, como o PHP.
    pub(crate) const REFRESH_TTL: Duration = Duration::from_secs(1_209_600);

    /// Se os cookies de sessão exigem HTTPS.
    pub(crate) const SECURE: bool = !cfg!(debug_assertions);

    /// A política `SameSite` dos cookies de sessão.
    ///
    /// `Strict` porque nenhuma navegação de terceiro precisa chegar autenticada:
    /// o front é uma aplicação só, e o que `Lax` liberaria é justamente o
    /// vetor de CSRF que não temos motivo para abrir.
    pub(crate) const SAME_SITE: SameSite = SameSite::Strict;
}
