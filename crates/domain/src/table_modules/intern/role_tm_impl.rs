//! A implementação das regras de papel.

use nutype::nutype;

use crate::domain::Role;
use crate::error::{FieldError, RoleError};
use crate::id::DatabaseIdGenerator;
use crate::table_modules::intern::models::role_model::RoleModel;
use crate::table_modules::RoleTM;

/// O nome de um papel.
#[nutype(sanitize(trim), validate(not_empty, len_char_max = 255))]
struct RoleName(String);

/// A implementação, genérica sobre o gerador de id.
#[derive(Clone)]
pub(crate) struct RoleTMImpl<G> {
    /// De onde sai a identidade de um papel novo.
    id_generator: G,
}

impl<G: DatabaseIdGenerator> RoleTMImpl<G> {
    /// Monta o `TableModule` com o seu gerador de id.
    pub(crate) const fn new(id_generator: G) -> Self {
        Self { id_generator }
    }
}

impl<G: DatabaseIdGenerator> RoleTM for RoleTMImpl<G> {
    fn create(&self, name: String, permissions: Vec<String>) -> Result<Box<dyn Role>, RoleError> {
        let name = RoleName::try_new(name)
            .map_err(|error| RoleError::Validation(vec![name_refused(&error)]))?;

        Ok(Box::new(RoleModel::new(
            self.id_generator.next(),
            name.into_inner(),
            permissions,
        )))
    }

    /// Substitui o conjunto de permissões do papel.
    ///
    /// Os slugs **não** são validados aqui: quem os produz é o registro de
    /// permissões do próprio sistema, não o cliente. Conferir se um slug existe
    /// é trabalho do `app`, que tem o catálogo à mão.
    fn update_permissions(
        &self,
        role: &dyn Role,
        permissions: Vec<String>,
    ) -> Result<Box<dyn Role>, RoleError> {
        let mut model = RoleModel::from_domain(role);
        model.set_permissions(permissions);
        Ok(Box::new(model))
    }
}

/// Comprimento máximo do nome, casando com a coluna `VARCHAR(255)`.
const MAX_NAME_LENGTH: usize = 255;

/// Traduz a recusa do nome na mensagem que o cliente lê.
fn name_refused(error: &RoleNameError) -> FieldError {
    match *error {
        RoleNameError::NotEmptyViolated => FieldError::new("name", "Name is required."),
        RoleNameError::LenCharMaxViolated => FieldError::new(
            "name",
            format!("Name must not exceed {MAX_NAME_LENGTH} characters."),
        ),
    }
}

#[cfg(test)]
#[path = "tests/role_tm_impl_test.rs"]
mod tests;
