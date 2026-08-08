//! A implementação das regras de papel.

use crate::error::{RoleError, Validation};
use crate::id::IntIdGenerator;
use crate::models::interno::role_model::RoleModel;
use crate::models::Role;
use crate::table_modules::RoleTM;

/// A implementação, genérica sobre o gerador de id.
pub(crate) struct RoleTMImpl<G> {
    /// De onde sai a identidade de um papel novo.
    id_generator: G,
}

impl<G: IntIdGenerator> RoleTMImpl<G> {
    /// Monta o `TableModule` com o seu gerador de id.
    pub(crate) const fn new(id_generator: G) -> Self {
        Self { id_generator }
    }

    /// Examina o nome, acumulando o que estiver errado.
    fn validate(name: &str) -> Validation {
        let mut errors = Validation::new();

        if name.trim().is_empty() {
            errors.add("name", "Name is required.");
        } else if name.chars().count() > MAX_NAME_LENGTH {
            errors.add(
                "name",
                format!("Name must not exceed {MAX_NAME_LENGTH} characters."),
            );
        }

        errors
    }
}

impl<G: IntIdGenerator> RoleTM for RoleTMImpl<G> {
    fn create(&self, name: String, permissions: Vec<String>) -> Result<Box<dyn Role>, RoleError> {
        Self::validate(&name)
            .into_result(())
            .map_err(RoleError::Validation)?;

        Ok(Box::new(RoleModel::new(
            self.id_generator.next(),
            name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_modules::interno::fields_of::fields_of;
    use pretty_assertions::assert_eq;

    struct FixedIdGenerator;
    impl IntIdGenerator for FixedIdGenerator {
        fn next(&self) -> String {
            "9Z8y".to_string()
        }
    }

    fn table_module() -> RoleTMImpl<FixedIdGenerator> {
        RoleTMImpl::new(FixedIdGenerator)
    }

    #[test]
    fn cria_papel_valido() {
        let role = table_module()
            .create("Operador".into(), vec!["container:read".into()])
            .expect("os dados são válidos");

        assert_eq!(role.id(), "9Z8y");
        assert_eq!(role.name(), "Operador");
        assert_eq!(role.permissions(), ["container:read"]);
    }

    #[test]
    fn recusa_nome_vazio_ou_longo_demais() {
        for bad in ["", "   ", &"x".repeat(MAX_NAME_LENGTH + 1)] {
            let error = table_module()
                .create(bad.into(), Vec::new())
                .err()
                .expect("nome inválido deve falhar");

            let RoleError::Validation(fields) = error;
            assert_eq!(fields_of(&fields), vec!["name"]);
        }
    }

    /// Um slug omitido é uma permissão **revogada**.
    ///
    /// Se isto virasse merge, não haveria como tirar uma permissão de um papel.
    #[test]
    fn atualizar_permissoes_substitui_em_vez_de_somar() {
        let role = table_module()
            .create("Operador".into(), vec!["a:read".into(), "a:write".into()])
            .expect("os dados são válidos");

        let updated = table_module()
            .update_permissions(role.as_ref(), vec!["a:read".into()])
            .expect("a substituição não valida slugs");

        assert_eq!(updated.permissions(), ["a:read"]);
        assert_eq!(role.permissions(), ["a:read", "a:write"]);
    }
}
