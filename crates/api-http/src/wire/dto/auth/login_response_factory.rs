//! O que `POST /auth/login` e `POST /setup` respondem.

use crate::error::api_error::ApiError;
use crate::wire::dto::auth::user_response_factory::UserResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;

/// Monta a resposta de uma sessão aberta.
///
/// O aninhamento é o que o planus já sabe fazer: `table()` devolve a tabela
/// *owned* com o `User` dentro, e o impl gerado de `WriteAsOffset` escreve o
/// filho antes do pai — que é, linha por linha, o `createUser` + `createLoginResponse`
/// que o PHP escrevia à mão.
pub(crate) struct LoginResponseFactory {
    /// O access token emitido.
    token: String,
    /// Como o token viaja — `cookie`, e não `Bearer`.
    token_type: &'static str,
    /// O dono da sessão, na forma enxuta que o login publica.
    user: UserResponseFactory,
}

impl LoginResponseFactory {
    /// Monta a factory com o token emitido e o dono dele.
    pub(crate) const fn new(
        token: String,
        token_type: &'static str,
        user: UserResponseFactory,
    ) -> Self {
        Self {
            token,
            token_type,
            user,
        }
    }
}

impl ResponseFactory for LoginResponseFactory {
    type Table = fbs::auth::LoginResponse;

    /// Monta a tabela, com a do `User` aninhada dentro.
    ///
    /// A factory filha produz a **tabela**, e o planus a escreve no lugar certo
    /// — é ele quem faz a coreografia de builder que o PHP escrevia à mão.
    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::auth::LoginResponse {
            token: Some(self.token.clone()),
            token_type: Some(self.token_type.to_owned()),
            user: Some(Box::new(self.user.table()?)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::strategy::encode_strategy::EncodeStrategy as _;
    use crate::wire::strategy::flatbuffers_encode_strategy::FlatBuffersEncodeStrategy;
    use crate::wire::strategy::json_encode_strategy::JsonEncodeStrategy;
    use pretty_assertions::assert_eq;

    /// Um `User` de domínio de mentira, sem passar pelo `TableModule`.
    ///
    /// As assinaturas são as do trait de domínio, e não escolha deste stub: o
    /// `&str` devolvido é emprestado do `&self` na trait, e mudá-lo aqui não
    /// compilaria.
    struct StubUser;

    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "a assinatura é a do trait de domínio: devolver &'static str não a satisfaz"
    )]
    impl portmaster_app::domain::User for StubUser {
        fn id(&self) -> &str {
            "aZ3"
        }
        fn name(&self) -> &str {
            "Ana"
        }
        fn email(&self) -> &str {
            "ana@portmaster.local"
        }
        fn password_hash(&self) -> &str {
            "$argon2id$não-deve-vazar"
        }
        fn roles(&self) -> &[Box<dyn portmaster_app::domain::Role>] {
            &[]
        }
        fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }
        fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }
        fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            None
        }
    }

    fn factory() -> LoginResponseFactory {
        LoginResponseFactory::new(
            "jwt.de.mentira".to_owned(),
            "cookie",
            UserResponseFactory::of(&StubUser),
        )
    }

    /// A suíte Go manda `Accept: application/x-flatbuffers` em toda requisição
    /// — ela não cobre JSON em ponto nenhum.
    ///
    /// Este `assert_eq!` da string exata é o **único** contrato verificável do
    /// caminho JSON, e a ordem dos campos faz parte dele: é a ordem de
    /// declaração do `.fbs`.
    #[test]
    fn o_json_bate_com_o_que_o_swagger_documenta() {
        let json = JsonEncodeStrategy
            .encode(&factory())
            .expect("a resposta precisa serializar");

        assert_eq!(
            String::from_utf8(json).expect("o JSON é UTF-8"),
            r#"{"token":"jwt.de.mentira","token_type":"cookie","user":{"id":"aZ3","name":"Ana","email":"ana@portmaster.local"}}"#
        );
    }

    /// A tabela do wire não tem onde pôr `password_hash`, e é isso que garante
    /// que ele não vaze.
    ///
    /// O teste existe para que a garantia continue sendo verdade se alguém
    /// acrescentar um campo ao `.fbs`.
    #[test]
    fn o_hash_da_senha_nao_atravessa() {
        let json = JsonEncodeStrategy
            .encode(&factory())
            .expect("a resposta precisa serializar");

        assert!(!String::from_utf8_lossy(&json).contains("argon2id"));
    }

    /// É o ponto do desenho: uma factory, duas strategies, nenhuma das duas
    /// sabendo o que a outra faz.
    #[test]
    fn a_mesma_factory_sai_nos_dois_formatos() {
        let binary = FlatBuffersEncodeStrategy
            .encode(&factory())
            .expect("a resposta precisa serializar");

        assert!(!binary.is_empty());
    }

    /// O `Renderable` devolve `Offset<()>`, e o `finish` escreve por ele.
    ///
    /// `Offset<T>::ALIGNMENT` é 4 para todo `T`, então apagar o tipo não muda
    /// um byte — este teste é o que sustenta essa afirmação.
    #[test]
    fn o_aninhamento_produz_os_mesmos_bytes_pelo_caminho_apagado() {
        let erased = FlatBuffersEncodeStrategy
            .encode(&factory())
            .expect("a resposta precisa serializar");

        let typed = {
            let mut builder = planus::Builder::new();
            let table = factory().table().expect("a tabela precisa montar");
            builder.finish(&table, None).to_vec()
        };

        assert_eq!(erased, typed);
    }
}
