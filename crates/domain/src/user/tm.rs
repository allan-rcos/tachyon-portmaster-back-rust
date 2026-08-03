//! Regras de usuário: o que é um nome, um e-mail e uma senha aceitáveis.

use crate::error::{UserError, Validation};
use crate::id::IntIdGenerator;
use crate::role::Role;
use crate::security::PasswordHasher;

use super::model::{User, UserModel};

/// Constrói e altera usuários, recusando-se a produzir um inválido.
///
/// Recebe **valores soltos**, nunca um `Command`: Command é vocabulário do
/// `app`, e o núcleo não o conhece.
pub trait UserTM {
    /// Cria um usuário novo, ainda não persistido.
    fn create(
        &self,
        name: String,
        email: String,
        password: String,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outro nome e e-mail.
    fn update(
        &self,
        user: &dyn User,
        name: String,
        email: String,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outra senha.
    ///
    /// Vale tanto para a troca feita pelo próprio dono quanto para a
    /// redefinição feita por um administrador: a regra sobre o que é uma senha
    /// aceitável é a mesma, e quem pode fazer o quê é decisão do `app`.
    fn change_password(
        &self,
        user: &dyn User,
        new_password: String,
    ) -> Result<Box<dyn User>, UserError>;

    /// Produz o usuário com outro conjunto de papéis.
    fn update_roles(
        &self,
        user: &dyn User,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError>;
}

/// Comprimento máximo do nome, casando com a coluna `VARCHAR(255)`.
const MAX_NAME_LENGTH: usize = 255;

/// Comprimento máximo do e-mail, casando com a coluna `VARCHAR(255)`.
const MAX_EMAIL_LENGTH: usize = 255;

/// Mínimo de caracteres numa senha.
const MIN_PASSWORD_LENGTH: usize = 8;

/// A implementação, genérica sobre os helpers que recebe.
///
/// Nem o gerador de id nem o hasher são instanciados aqui: chegam injetados pelo
/// factory do provider, o que os torna substituíveis em teste sem que nada além
/// do domínio saiba que existem.
pub(crate) struct UserTMImpl<G, H> {
    id_generator: G,
    password_hasher: H,
}

impl<G: IntIdGenerator, H: PasswordHasher> UserTMImpl<G, H> {
    /// Monta o TableModule com os seus helpers.
    pub(crate) fn new(id_generator: G, password_hasher: H) -> Self {
        Self {
            id_generator,
            password_hasher,
        }
    }

    /// Examina nome e e-mail, acumulando tudo que estiver errado.
    fn validate_profile(&self, name: &str, email: &str, errors: &mut Validation) {
        if name.trim().is_empty() {
            errors.add("name", "Name is required");
        } else if name.chars().count() > MAX_NAME_LENGTH {
            errors.add(
                "name",
                format!("Name must not exceed {MAX_NAME_LENGTH} characters"),
            );
        }

        if email.trim().is_empty() {
            errors.add("email", "Email is required");
        } else if email.chars().count() > MAX_EMAIL_LENGTH {
            errors.add(
                "email",
                format!("Email must not exceed {MAX_EMAIL_LENGTH} characters"),
            );
        } else if !is_plausible_email(email) {
            errors.add("email", "Invalid email format");
        }
    }

    /// Examina a senha.
    ///
    /// Exige minúscula, maiúscula, dígito e comprimento mínimo. A mensagem
    /// enumera as quatro condições de uma vez porque devolver "falta um dígito",
    /// depois "falta uma maiúscula", faria o usuário descobrir a regra por
    /// tentativa e erro.
    fn validate_password(&self, password: &str, errors: &mut Validation) {
        let long_enough = password.chars().count() >= MIN_PASSWORD_LENGTH;
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        errors.add_if(
            !(long_enough && has_lowercase && has_uppercase && has_digit),
            "password",
            "Password must be at least 8 characters long and include uppercase, \
             lowercase letters, and numbers",
        );
    }

    /// Produz o usuário com outra senha, validando-a antes.
    fn with_password(
        &self,
        user: &dyn User,
        new_password: String,
    ) -> Result<Box<dyn User>, UserError> {
        let mut errors = Validation::new();
        self.validate_password(&new_password, &mut errors);
        errors.into_result(()).map_err(UserError::Validation)?;

        let mut model = UserModel::from_domain(user);
        model.set_password_hash(self.password_hasher.hash(&new_password));
        Ok(Box::new(model))
    }
}

impl<G: IntIdGenerator, H: PasswordHasher> UserTM for UserTMImpl<G, H> {
    fn create(
        &self,
        name: String,
        email: String,
        password: String,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError> {
        let mut errors = Validation::new();
        self.validate_profile(&name, &email, &mut errors);
        self.validate_password(&password, &mut errors);
        errors.into_result(()).map_err(UserError::Validation)?;

        // A senha em claro morre aqui: o que segue para o repositório é o hash.
        let password_hash = self.password_hasher.hash(&password);

        Ok(Box::new(UserModel::new(
            self.id_generator.next(),
            name,
            email,
            password_hash,
            roles,
        )))
    }

    fn update(
        &self,
        user: &dyn User,
        name: String,
        email: String,
    ) -> Result<Box<dyn User>, UserError> {
        let mut errors = Validation::new();
        self.validate_profile(&name, &email, &mut errors);
        errors.into_result(()).map_err(UserError::Validation)?;

        let mut model = UserModel::from_domain(user);
        model.set_profile(name, email);
        Ok(Box::new(model))
    }

    fn change_password(
        &self,
        user: &dyn User,
        new_password: String,
    ) -> Result<Box<dyn User>, UserError> {
        self.with_password(user, new_password)
    }

    fn update_roles(
        &self,
        user: &dyn User,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError> {
        let mut model = UserModel::from_domain(user);
        model.set_roles(roles);
        Ok(Box::new(model))
    }
}

/// Checagem estrutural de e-mail.
///
/// Deliberadamente frouxa. Validar e-mail por regex é uma armadilha conhecida —
/// a gramática real do RFC 5322 aceita coisas que quase toda regex recusa — e o
/// único teste que de fato prova um endereço é mandar uma mensagem para ele. O
/// que se pega aqui é erro de digitação óbvio.
fn is_plausible_email(email: &str) -> bool {
    let mut parts = email.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };

    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
}

/// Atalho para ler os campos de um lote de erros de validação.
#[cfg(test)]
pub(crate) fn fields_of(errors: &[crate::error::FieldError]) -> Vec<&str> {
    errors.iter().map(|e| e.field.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::argon2::Argon2Hasher;
    use pretty_assertions::assert_eq;

    /// Gerador determinístico: o teste não deve depender de relógio nem de sorte.
    struct FixedIdGenerator;
    impl IntIdGenerator for FixedIdGenerator {
        fn next(&self) -> String {
            "1A2b3C".to_string()
        }
    }

    fn table_module() -> UserTMImpl<FixedIdGenerator, Argon2Hasher> {
        UserTMImpl::new(FixedIdGenerator, Argon2Hasher::new())
    }

    fn valid_user() -> Box<dyn User> {
        table_module()
            .create(
                "Ana".into(),
                "ana@portmaster.local".into(),
                "Portmaster1".into(),
                Vec::new(),
            )
            .expect("os dados do fixture são válidos")
    }

    #[test]
    fn cria_usuario_valido_sem_guardar_a_senha() {
        let user = valid_user();

        assert_eq!(user.id(), "1A2b3C");
        assert_eq!(user.name(), "Ana");
        assert_eq!(user.email(), "ana@portmaster.local");
        assert_ne!(user.password_hash(), "Portmaster1");
        assert!(user.password_hash().starts_with("$argon2"));
        assert_eq!(user.deleted_at(), None);
    }

    #[test]
    fn acumula_todos_os_campos_invalidos_de_uma_vez() {
        // O ponto do lote: quem enviou três campos errados descobre os três
        // agora, não um por requisição.
        let error = table_module()
            .create("".into(), "sem-arroba".into(), "curta".into(), Vec::new())
            .err()
            .expect("nome vazio, e-mail inválido e senha fraca devem falhar");

        let UserError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["name", "email", "password"]);
    }

    #[test]
    fn recusa_senha_sem_a_variedade_exigida() {
        for weak in ["portmaster1", "PORTMASTER1", "PortmasterX", "Pm1"] {
            let error = table_module()
                .create("Ana".into(), "ana@x.com".into(), weak.into(), Vec::new())
                .err()
                .expect("{weak} deveria ser recusada");

            let UserError::Validation(fields) = error;
            assert_eq!(fields_of(&fields), vec!["password"], "senha: {weak}");
        }
    }

    #[test]
    fn recusa_email_malformado() {
        for bad in [
            "sem-arroba",
            "@dominio.com",
            "ana@",
            "ana@dominio",
            "a b@c.com",
        ] {
            let error = table_module()
                .create("Ana".into(), bad.into(), "Portmaster1".into(), Vec::new())
                .err()
                .expect("{bad} deveria ser recusado");

            let UserError::Validation(fields) = error;
            assert_eq!(fields_of(&fields), vec!["email"], "e-mail: {bad}");
        }
    }

    #[test]
    fn update_nao_altera_o_usuario_recebido() {
        // A transição produz um objeto novo. Se ela mutasse o argumento, uma
        // atualização recusada mais adiante deixaria o chamador com um objeto
        // meio-alterado.
        let original = valid_user();
        let updated = table_module()
            .update(
                original.as_ref(),
                "Ana Maria".into(),
                "ana.maria@x.com".into(),
            )
            .expect("os dados são válidos");

        assert_eq!(original.name(), "Ana");
        assert_eq!(updated.name(), "Ana Maria");
        assert_eq!(updated.id(), original.id());
    }

    #[test]
    fn troca_de_senha_preserva_o_resto_e_muda_o_hash() {
        let original = valid_user();
        let changed = table_module()
            .change_password(original.as_ref(), "Portmaster2".into())
            .expect("a senha nova é válida");

        assert_eq!(changed.email(), original.email());
        assert_ne!(changed.password_hash(), original.password_hash());
    }

    #[test]
    fn troca_de_senha_valida_a_nova() {
        let original = valid_user();
        let error = table_module()
            .change_password(original.as_ref(), "fraca".into())
            .err()
            .expect("senha fraca deve ser recusada também na troca");

        let UserError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["password"]);
    }
}
