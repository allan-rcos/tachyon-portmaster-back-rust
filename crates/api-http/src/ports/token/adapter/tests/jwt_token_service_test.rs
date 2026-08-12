//! Os testes de `jwt_token_service`.

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
        issuer: "tachyon/portmaster".to_owned(),
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
