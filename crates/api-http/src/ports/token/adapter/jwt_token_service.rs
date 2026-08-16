//! O token de sessão como JWT. Não sai do módulo.
//!
//! ## O principal viaja como `FlatBuffers` dentro da claim
//!
//! O JWT não carrega claims soltas por usuário. O principal inteiro — id, nome,
//! e-mail, papéis e permissões — é serializado na tabela `TokenUser` de
//! `token.fbs`, codificado em base64url e posto numa claim só. Duas razões: o
//! token fica menor que o JSON equivalente, e o payload deixa de ser legível
//! para quem espia o cookie por curiosidade.
//!
//! Não é sigilo — base64 não esconde nada de quem queira ler. O que a assinatura
//! garante é **integridade**: o conteúdo não pode ser alterado sem o segredo.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use portmaster_app::context::{RoleContext, UserContext};
use portmaster_app::domain::User;
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};

use crate::config::jwt_config::JwtConfig;
use crate::ports::error::api_error::ApiError;
use crate::ports::session_policy::SessionPolicy;
use crate::ports::token::token_service::TokenService;
use crate::wire::tables as fbs;

/// As claims do access token.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Id do usuário, em base62.
    sub: String,
    /// Quem emitiu.
    iss: String,
    /// Quando expira, em epoch de segundos.
    exp: u64,
    /// Quando foi emitido.
    iat: u64,
    /// O principal inteiro, em `FlatBuffers` e base64url.
    user_data: String,
}

/// Monta o serviço de token a partir da configuração.
///
/// Devolve o tipo concreto porque o provider precisa guardá-lo num `static`, e
/// um `static` exige um tipo com nome — um `impl Trait` não é um. Nem a função
/// nem o tipo saem do módulo `ports::token`, e quem entrega o contrato é o
/// provider.
pub(in crate::ports::token) fn jwt_token_service(config: &JwtConfig) -> JwtTokenService {
    let secret = config.secret.expose_secret().as_bytes();

    JwtTokenService {
        encoding: EncodingKey::from_secret(secret),
        decoding: DecodingKey::from_secret(secret),
        issuer: config.issuer.clone(),
    }
}

/// Emite e confere tokens assinados em HS256.
#[derive(Clone)]
pub(in crate::ports::token) struct JwtTokenService {
    /// A chave de assinatura.
    encoding: EncodingKey,
    /// A chave de verificação.
    decoding: DecodingKey,
    /// Quem emitiu, gravado e conferido na claim `iss`.
    issuer: String,
}

impl JwtTokenService {
    /// O agora, em epoch de segundos.
    ///
    /// `Utc::now()` direto, e não um relógio injetado: quem confere o `exp` é o
    /// `jsonwebtoken`, contra o relógio de parede real, e não há onde injetar
    /// nada ali. Um relógio trocável na emissão e fixo na conferência não tornava
    /// a expiração testável — só dava essa impressão.
    fn epoch_seconds() -> u64 {
        u64::try_from(Utc::now().timestamp()).unwrap_or_default()
    }

    /// Serializa o principal na tabela `TokenUser`, em base64url.
    ///
    /// Método privado e não linha solta no [`Self::issue`]: são vinte linhas de
    /// coreografia de builder cujo ponto — o que entra no token e o que fica de
    /// fora — merece o nome e a explicação acima.
    fn pack_principal(user: &dyn User) -> String {
        let token_user = fbs::token::TokenUser {
            id: Some(user.id().to_owned()),
            name: Some(user.name().to_owned()),
            email: Some(user.email().to_owned()),
            roles: Some(
                user.roles()
                    .iter()
                    .map(|role| fbs::token::TokenRole {
                        id: Some(role.id().to_owned()),
                        name: Some(role.name().to_owned()),
                        permissions: Some(role.permissions().to_vec()),
                    })
                    .collect(),
            ),
        };

        let mut builder = planus::Builder::new();

        URL_SAFE_NO_PAD.encode(builder.finish(&token_user, None))
    }

    /// Desfaz o [`Self::pack_principal`].
    ///
    /// Tolera todo campo ausente menos os ids: um papel sem id não identifica
    /// nada, e um principal sem id não é ninguém. Nome e permissões vazios
    /// degradam a sessão sem invalidá-la.
    fn unpack_principal(packed: &str) -> Option<UserContext> {
        use planus::ReadAsRoot as _;

        let bytes = URL_SAFE_NO_PAD.decode(packed).ok()?;
        let token_user = fbs::token::TokenUserRef::read_as_root(&bytes).ok()?;

        let roles = token_user
            .roles()
            .ok()
            .flatten()
            .map(|roles| {
                roles
                    .iter()
                    .filter_map(|role| {
                        let role = role.ok()?;

                        Some(RoleContext {
                            id: role.id().ok().flatten()?.to_owned(),
                            name: role.name().ok().flatten().unwrap_or_default().to_owned(),
                            permissions: role
                                .permissions()
                                .ok()
                                .flatten()
                                .map(|slugs| {
                                    slugs
                                        .iter()
                                        .filter_map(|slug| Some(slug.ok()?.to_owned()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(UserContext {
            id: token_user.id().ok().flatten()?.to_owned(),
            name: token_user
                .name()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            email: token_user
                .email()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            roles,
        })
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, user: &dyn User) -> Result<String, ApiError> {
        let now = Self::epoch_seconds();

        #[allow(
            clippy::arithmetic_side_effects,
            reason = "epoch em segundos mais um TTL de minutos: estourar u64 são ~500 bilhões de anos"
        )]
        let claims = Claims {
            sub: user.id().to_owned(),
            iss: self.issuer.clone(),
            exp: now + SessionPolicy::ACCESS_TTL.as_secs(),
            iat: now,
            user_data: Self::pack_principal(user),
        };

        jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &self.encoding).map_err(|e| {
            ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
    }

    /// Confere a assinatura e devolve o principal que o token carrega.
    ///
    /// ## Sem tolerância de relógio
    ///
    /// O padrão da lib são 60 segundos, pensados para quando quem assina e quem
    /// confere são máquinas diferentes — aqui são o mesmo processo, com o mesmo
    /// relógio. Mantê-la só faria todo token valer um minuto a mais do que o
    /// `exp` que ele mesmo declara.
    fn verify(&self, token: &str) -> Option<UserContext> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);

        validation.leeway = 0;

        let claims = jsonwebtoken::decode::<Claims>(token, &self.decoding, &validation)
            .ok()?
            .claims;

        Self::unpack_principal(&claims.user_data)
    }
}
