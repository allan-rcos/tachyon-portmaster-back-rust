//! Uma permissão de domínio, montada para o teste.

use portmaster_domain::domain::Permission;

/// Uma permissão que o teste controla.
///
/// O `PermissionModel` do `domain` é interno lá — só o `PermissionTM` o
/// constrói. Como aqui o table module é mock, alguém precisa produzir o
/// `Box<dyn Permission>` que ele devolveria.
pub(crate) struct StubPermission {
    /// O slug, em `recurso:ação`.
    slug: String,
}

impl StubPermission {
    /// A permissão deste slug, dentro do `Box` que o table module devolveria.
    pub(crate) fn boxed(slug: &str) -> Box<dyn Permission> {
        Box::new(Self {
            slug: slug.to_owned(),
        })
    }
}

impl Permission for StubPermission {
    fn slug(&self) -> &str {
        &self.slug
    }
}
