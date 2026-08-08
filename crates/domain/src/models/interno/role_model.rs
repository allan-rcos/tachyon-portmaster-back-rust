//! A implementação de domínio de `Role`.

use chrono::{DateTime, Utc};

use crate::models::Role;

/// A implementação do domínio de [`Role`].
pub struct RoleModel {
    /// Identidade, em base62.
    id: String,
    /// Nome do papel, como a administração o exibe.
    name: String,
    /// Os slugs que este papel concede.
    permissions: Vec<String>,
    /// Quando foi criado, em UTC.
    created_at: DateTime<Utc>,
    /// Quando mudou pela última vez; o `set_*` o move.
    updated_at: DateTime<Utc>,
    /// Quando foi removido, ou `None` se ativo — o soft-delete.
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
