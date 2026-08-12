//! A implementação das regras de usuário.

use nutype::nutype;

use crate::domain::{Role, User};
use crate::error::{FieldError, UserError};
use crate::id::DatabaseIdGenerator;
use crate::security::PasswordHasher;
use crate::table_modules::intern::models::user_model::UserModel;
use crate::table_modules::UserTM;

/// O nome de uma pessoa.
///
/// O `trim` não é cosmético: sem ele `"   "` passaria por "preenchido", e o que
/// iria para a coluna seria um nome que ninguém consegue procurar.
#[nutype(sanitize(trim), validate(not_empty, len_char_max = 255))]
struct PersonName(String);

/// O e-mail de um usuário.
///
/// Além de obrigatório e do tamanho da coluna, precisa passar pela checagem
/// estrutural de [`is_plausible_email`].
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 255, predicate = is_plausible_email)
)]
struct EmailAddress(String);

/// Uma senha em claro, antes de virar hash.
///
/// Sem `sanitize`: espaço numa senha é caractere como outro qualquer, e apará-lo
/// mudaria em silêncio o que o usuário digitou.
#[nutype(validate(predicate = is_strong_password))]
struct Password(String);

/// A implementação, genérica sobre os helpers que recebe.
///
/// Nem o gerador de id nem o hasher são instanciados aqui: chegam injetados pelo
/// factory do provider, o que os torna substituíveis em teste sem que nada além
/// do domínio saiba que existem.
#[derive(Clone)]
pub(crate) struct UserTMImpl<G, H> {
    /// De onde sai a identidade de um usuário novo.
    id_generator: G,
    /// Quem transforma a senha em hash — lento de propósito.
    password_hasher: H,
}

impl<G: DatabaseIdGenerator, H: PasswordHasher> UserTMImpl<G, H> {
    /// Monta o `TableModule` com os seus helpers.
    pub(crate) const fn new(id_generator: G, password_hasher: H) -> Self {
        Self {
            id_generator,
            password_hasher,
        }
    }

    /// Produz o usuário com outra senha, validando-a antes.
    fn with_password(
        &self,
        user: &dyn User,
        new_password: String,
    ) -> Result<Box<dyn User>, UserError> {
        let password =
            Password::try_new(new_password).map_err(|_| UserError::Validation(vec![weak()]))?;

        let mut model = UserModel::from_domain(user);
        model.set_password_hash(self.password_hasher.hash(password.into_inner().as_str()));
        Ok(Box::new(model))
    }
}

impl<G: DatabaseIdGenerator, H: PasswordHasher> UserTM for UserTMImpl<G, H> {
    /// Cria um usuário, recusando de uma vez tudo que estiver errado.
    ///
    /// Os três `try_new` acontecem **antes** de qualquer retorno: é o que faz
    /// quem enviou nome vazio, e-mail torto e senha fraca descobrir os três
    /// agora, em vez de um por requisição.
    fn create(
        &self,
        name: String,
        email: String,
        password: String,
        roles: Vec<Box<dyn Role>>,
    ) -> Result<Box<dyn User>, UserError> {
        let checked_name = PersonName::try_new(name);
        let checked_email = EmailAddress::try_new(email);
        let checked_password = Password::try_new(password);

        let mut errors = Vec::new();
        if let Err(error) = &checked_name {
            errors.push(name_refused(error));
        }
        if let Err(error) = &checked_email {
            errors.push(email_refused(error));
        }
        if checked_password.is_err() {
            errors.push(weak());
        }

        let (Ok(name), Ok(email), Ok(password)) = (checked_name, checked_email, checked_password)
        else {
            return Err(UserError::Validation(errors));
        };

        // A senha em claro morre aqui: o que segue para o repositório é o hash.
        let password_hash = self.password_hasher.hash(password.into_inner().as_str());

        Ok(Box::new(UserModel::new(
            self.id_generator.next(),
            name.into_inner(),
            email.into_inner(),
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
        let checked_name = PersonName::try_new(name);
        let checked_email = EmailAddress::try_new(email);

        let mut errors = Vec::new();
        if let Err(error) = &checked_name {
            errors.push(name_refused(error));
        }
        if let Err(error) = &checked_email {
            errors.push(email_refused(error));
        }

        let (Ok(name), Ok(email)) = (checked_name, checked_email) else {
            return Err(UserError::Validation(errors));
        };

        let mut model = UserModel::from_domain(user);
        model.set_profile(name.into_inner(), email.into_inner());
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

/// Comprimento máximo do nome, casando com a coluna `VARCHAR(255)`.
const MAX_NAME_LENGTH: usize = 255;

/// Comprimento máximo do e-mail, casando com a coluna `VARCHAR(255)`.
const MAX_EMAIL_LENGTH: usize = 255;

/// Mínimo de caracteres numa senha.
const MIN_PASSWORD_LENGTH: usize = 8;

/// Traduz a recusa do nome na mensagem que o cliente lê.
fn name_refused(error: &PersonNameError) -> FieldError {
    match *error {
        PersonNameError::NotEmptyViolated => FieldError::new("name", "Name is required"),
        PersonNameError::LenCharMaxViolated => FieldError::new(
            "name",
            format!("Name must not exceed {MAX_NAME_LENGTH} characters"),
        ),
    }
}

/// Traduz a recusa do e-mail na mensagem que o cliente lê.
fn email_refused(error: &EmailAddressError) -> FieldError {
    match *error {
        EmailAddressError::NotEmptyViolated => FieldError::new("email", "Email is required"),
        EmailAddressError::LenCharMaxViolated => FieldError::new(
            "email",
            format!("Email must not exceed {MAX_EMAIL_LENGTH} characters"),
        ),
        EmailAddressError::PredicateViolated => FieldError::new("email", "Invalid email format"),
    }
}

/// A recusa de uma senha.
///
/// A mensagem enumera as quatro condições de uma vez porque devolver "falta um
/// dígito", depois "falta uma maiúscula", faria o usuário descobrir a regra por
/// tentativa e erro.
fn weak() -> FieldError {
    FieldError::new(
        "password",
        "Password must be at least 8 characters long and include uppercase, \
         lowercase letters, and numbers",
    )
}

/// Exige minúscula, maiúscula, dígito e comprimento mínimo.
fn is_strong_password(password: &str) -> bool {
    password.chars().count() >= MIN_PASSWORD_LENGTH
        && password.chars().any(|c| c.is_ascii_lowercase())
        && password.chars().any(|c| c.is_ascii_uppercase())
        && password.chars().any(|c| c.is_ascii_digit())
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

#[cfg(test)]
#[path = "tests/user_tm_impl_test.rs"]
mod tests;
