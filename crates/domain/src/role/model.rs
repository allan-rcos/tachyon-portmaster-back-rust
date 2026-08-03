//! O papel: um conjunto nomeado de permissões, concedido a usuários.

use chrono::{DateTime, Utc};

/// Um pacote de permissões que se concede a um usuário.
///
/// O trait é somente-leitura, como todo objeto de domínio.
pub trait Role: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Nome de exibição.
    fn name(&self) -> &str;

    /// Slugs de permissão que este papel concede, em `domain:action`.
    ///
    /// Slugs e não objetos `Permission`: o papel é persistido como JSON e
    /// sobrevive a qualquer registro em memória, cujos ids numéricos são
    /// reatribuídos a cada boot. O slug é a única referência estável.
    fn permissions(&self) -> &[String];

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Duplica o papel atrás do trait.
    ///
    /// Existe porque um usuário carrega os seus papéis como `Box<dyn Role>` e
    /// precisa ser reconstruível a partir do trait: sem isto, recriar um usuário
    /// exigiria conhecer o tipo concreto por trás de cada papel, que é
    /// exatamente o que o trait esconde.
    fn clone_role(&self) -> Box<dyn Role>;
}

/// A implementação do domínio de [`Role`].
pub(crate) struct RoleModel {
    id: String,
    name: String,
    permissions: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl RoleModel {
    /// Monta um papel a partir de campos já validados.
    pub(crate) fn new(id: String, name: String, permissions: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            permissions,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Recria o model a partir de qualquer [`Role`].
    pub(crate) fn from_domain(source: &dyn Role) -> Self {
        Self {
            id: source.id().to_owned(),
            name: source.name().to_owned(),
            permissions: source.permissions().to_vec(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        }
    }

    /// Substitui a lista de permissões, marcando a alteração.
    ///
    /// Substitui em vez de somar: um slug omitido é uma permissão revogada.
    pub(crate) fn set_permissions(&mut self, permissions: Vec<String>) {
        self.permissions = permissions;
        self.updated_at = Utc::now();
    }
}

impl Role for RoleModel {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn permissions(&self) -> &[String] {
        &self.permissions
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

    fn clone_role(&self) -> Box<dyn Role> {
        Box::new(Self {
            id: self.id.clone(),
            name: self.name.clone(),
            permissions: self.permissions.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        })
    }
}
