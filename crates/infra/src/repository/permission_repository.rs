//! O contrato de persistência de permission.

use portmaster_domain::domain::Permission;

/// Registro de permissões.
///
/// O backing é **cache**, nunca o banco: o catálogo é preenchido em código no
/// boot, é imutável depois disso, e é lido a cada verificação de autorização.
#[trait_variant::make(Send)]
pub trait PermissionRepository {
    /// Registra uma permissão. Idempotente por slug.
    async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()>;

    /// Todos os slugs registrados.
    async fn all(&self) -> anyhow::Result<Vec<String>>;

    /// Se um slug foi registrado.
    async fn has(&self, slug: &str) -> anyhow::Result<bool>;
}
