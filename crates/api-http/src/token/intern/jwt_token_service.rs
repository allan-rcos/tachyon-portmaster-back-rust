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
use std::time::Duration;

use crate::config::jwt_config::JwtConfig;
use crate::error::api_error::ApiError;
use crate::token::token_service::TokenService;
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

/// Emite e confere tokens assinados em HS256.
#[derive(Clone)]
pub(crate) struct JwtTokenService {
    /// A chave de assinatura.
    encoding: EncodingKey,
    /// A chave de verificação.
    decoding: DecodingKey,
    /// Quem emitiu, gravado e conferido na claim `iss`.
    issuer: String,
    /// Validade do access token.
    ttl: Duration,
}

impl JwtTokenService {
    /// Monta o serviço a partir da configuração.
    pub(crate) fn new(config: &JwtConfig) -> Self {
        let secret = config.secret.expose_secret().as_bytes();

        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            issuer: config.issuer.clone(),
            ttl: config.ttl,
        }
    }

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
            exp: now + self.ttl.as_secs(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use portmaster_app::domain::Role;
    use portmaster_app::SecretString;
    use pretty_assertions::assert_eq;

    /// Um papel de mentira, para montar o principal.
    struct StubRole;

    impl Role for StubRole {
        fn id(&self) -> &'static str {
            "r1"
        }
        fn name(&self) -> &'static str {
            "Operador"
        }
        fn permissions(&self) -> &[String] {
            // Um slice estático não serve porque o trait pede `&[String]`.
            static PERMISSIONS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
            PERMISSIONS.get_or_init(|| vec!["container:seal".to_owned()])
        }
        fn created_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn updated_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn deleted_at(&self) -> Option<DateTime<Utc>> {
            None
        }
        fn clone_role(&self) -> Box<dyn Role> {
            Box::new(Self)
        }
    }

    /// Um usuário de mentira.
    struct StubUser {
        roles: Vec<Box<dyn Role>>,
    }

    impl User for StubUser {
        fn id(&self) -> &'static str {
            "u1"
        }
        fn name(&self) -> &'static str {
            "Ana"
        }
        fn email(&self) -> &'static str {
            "ana@portmaster.local"
        }
        fn password_hash(&self) -> &'static str {
            "$argon2id$nunca-deve-sair-daqui"
        }
        fn roles(&self) -> &[Box<dyn Role>] {
            &self.roles
        }
        fn created_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn updated_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn deleted_at(&self) -> Option<DateTime<Utc>> {
            None
        }
    }

    fn config_base() -> JwtConfig {
        JwtConfig {
            secret: SecretString::from("dev-only-change-me-32-bytes-minimum"),
            ttl: Duration::from_secs(3600),
            issuer: "tachyon/portmaster".to_owned(),
            cookie_name: "auth_token".to_owned(),
            cookie_secure: false,
            cookie_same_site: "Strict".to_owned(),
            refresh_cookie_name: "refresh_token".to_owned(),
            refresh_ttl: Duration::from_secs(1_209_600),
        }
    }

    fn service() -> JwtTokenService {
        JwtTokenService::new(&config_base())
    }

    fn user() -> StubUser {
        StubUser {
            roles: vec![Box::new(StubRole)],
        }
    }

    /// É o que sustenta a autenticação stateless: tudo que a autorização
    /// precisa saber viaja no token, e nenhum middleware toca o banco.
    #[test]
    fn o_token_devolve_o_principal_inteiro() {
        let service = service();
        let token = service.issue(&user()).expect("o token deveria ser emitido");

        let context = service
            .verify(&token)
            .expect("token recém-emitido deveria valer");

        assert_eq!(context.id, "u1");
        assert_eq!(context.email, "ana@portmaster.local");
        assert_eq!(context.roles.len(), 1);
        assert!(context.has_permission("container:seal"));
    }

    /// O trait `User` o expõe, e empacotar o usuário inteiro por descuido o
    /// mandaria para o navegador dentro de um cookie.
    #[test]
    fn o_hash_da_senha_nao_entra_no_token() {
        let token = service()
            .issue(&user())
            .expect("o token deveria ser emitido");

        assert!(
            !token.contains("argon2"),
            "o hash da senha vazou para o token"
        );
    }

    #[test]
    fn um_token_de_outro_segredo_e_recusado() {
        let token = service()
            .issue(&user())
            .expect("o token deveria ser emitido");

        let outro = JwtTokenService::new(&JwtConfig {
            secret: SecretString::from("outro-segredo-de-32-bytes-no-minimo"),
            ..config_base()
        });

        assert!(outro.verify(&token).is_none());
    }

    #[test]
    fn um_token_de_outro_emissor_e_recusado() {
        let alheio = JwtTokenService::new(&JwtConfig {
            issuer: "outro/sistema".to_owned(),
            ..config_base()
        });
        let token = alheio.issue(&user()).expect("o token deveria ser emitido");

        assert!(service().verify(&token).is_none());
    }

    /// O `exp` é forjado no passado em vez de emitido por um relógio de teste.
    ///
    /// Quem confere o `exp` é o `jsonwebtoken`, contra o relógio de parede real:
    /// não há como recuar o relógio da conferência, então o que se recua é a
    /// claim. Assinado com o segredo de verdade, é o token que um cliente
    /// apresentaria uma hora depois de recebê-lo.
    #[test]
    fn um_token_expirado_e_recusado() {
        let service = service();
        let issued_at = Utc::now().timestamp() - 7200;

        let claims = Claims {
            sub: "u1".to_owned(),
            iss: config_base().issuer,
            exp: u64::try_from(issued_at + 3600).expect("o epoch do teste é positivo"),
            iat: u64::try_from(issued_at).expect("o epoch do teste é positivo"),
            user_data: JwtTokenService::pack_principal(&user()),
        };

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(config_base().secret.expose_secret().as_bytes()),
        )
        .expect("o token forjado deveria ser assinado");

        assert!(service.verify(&token).is_none());
    }

    #[test]
    fn lixo_no_lugar_do_token_nao_entra_em_panico() {
        for entrada in ["", "nao.e.um.jwt", "a.b", "....."] {
            assert!(service().verify(entrada).is_none());
        }
    }
}
