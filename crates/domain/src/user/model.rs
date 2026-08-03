//! O usuário: quem entra no sistema, e os papéis que decidem o que pode fazer.

use chrono::{DateTime, Utc};

use crate::role::Role;

/// Alguém que faz login, com os papéis que carrega.
///
/// Permissão nunca é concedida direto a uma pessoa — ela vem pelo papel, e é a
/// lista de slugs do papel que o guarda de autorização confere.
///
/// O trait é **somente-leitura**: só existem getters. Quem o recebe consegue ler
/// o usuário mas não alterá-lo, e é isso que impede o `app` ou o `api` de
/// mudarem um e-mail sem passar pela validação do TableModule.
pub trait User: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Nome de exibição.
    fn name(&self) -> &str;

    /// E-mail, que é também o identificador de login.
    fn email(&self) -> &str;

    /// Hash Argon2id da senha — nunca a senha.
    ///
    /// A senha em claro atravessa só o TableModule, o tempo de ser validada e
    /// hasheada; nada mais no sistema a retém.
    fn password_hash(&self) -> &str;

    /// Papéis atribuídos, na ordem em que foram concedidos.
    fn roles(&self) -> &[Box<dyn Role>];

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    ///
    /// Usuário é entidade forte: remover grava a data em vez de apagar a linha,
    /// e toda leitura filtra por `None`.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
}

/// A implementação do domínio de [`User`].
///
/// Construída e alterada **apenas** pelo [`UserTM`](crate::user::UserTM), que é
/// quem conhece as regras. Nem o UseCase nem o repositório a instanciam.
pub(crate) struct UserModel {
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
    /// É o que permite ao TableModule produzir a versão alterada de um usuário
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
