//! A implementação de domínio de `User`.

use chrono::{DateTime, Utc};

use crate::models::{Role, User};

/// A implementação do domínio de [`User`].
///
/// Construída e alterada **apenas** pelo [`UserTM`](crate::table_modules::UserTM), que é
/// quem conhece as regras. Nem o `UseCase` nem o repositório a instanciam.
pub struct UserModel {
    id: String,
    name: String,
    email: String,
    password_hash: String,
    roles: Vec<Box<dyn Role>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl UserModel {
    /// Monta um usuário a partir de campos já validados.
    pub(crate) fn new(
        id: String,
        name: String,
        email: String,
        password_hash: String,
        roles: Vec<Box<dyn Role>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            email,
            password_hash,
            roles,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Recria o model a partir de qualquer [`User`].
    ///
    /// É o que permite ao `TableModule` produzir a versão alterada de um usuário
    /// que chegou como trait read-only: ele não pode editar o objeto recebido,
    /// então constrói outro.
    pub(crate) fn from_domain(source: &dyn User) -> Self {
        Self {
            id: source.id().to_owned(),
            name: source.name().to_owned(),
            email: source.email().to_owned(),
            password_hash: source.password_hash().to_owned(),
            roles: source.roles().iter().map(|r| r.clone_role()).collect(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        }
    }

    /// Substitui nome e e-mail, marcando a alteração.
    pub(crate) fn set_profile(&mut self, name: String, email: String) {
        self.name = name;
        self.email = email;
        self.updated_at = Utc::now();
    }

    /// Substitui o hash da senha, marcando a alteração.
    pub(crate) fn set_password_hash(&mut self, password_hash: String) {
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
    }

    /// Substitui os papéis, marcando a alteração.
    ///
    /// Substitui em vez de somar: um papel omitido é um papel revogado.
    pub(crate) fn set_roles(&mut self, roles: Vec<Box<dyn Role>>) {
        self.roles = roles;
        self.updated_at = Utc::now();
    }
}

impl User for UserModel {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }

    fn roles(&self) -> &[Box<dyn Role>] {
        &self.roles
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}
