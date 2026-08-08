//! Os slugs de permissão do sistema.
//!
//! São **contrato**: já existem em papéis gravados no banco de quem roda a
//! versão PHP. Renomear qualquer um destes revoga silenciosamente o acesso de
//! quem o tinha.
//!
//! Namespace no molde do `Base62`: o nome do tipo é o que dá sentido às
//! constantes, e `PermissionSlug::CONTAINER_CREATE` diz de onde o valor vem.

/// Os slugs de permissão, literalmente como o PHP os tinha.
pub struct PermissionSlug;

impl PermissionSlug {
    /// Registrar um contêiner.
    pub const CONTAINER_CREATE: &str = "container:create";
    /// Remover um contêiner.
    pub const CONTAINER_DELETE: &str = "container:delete";
    /// Despachar um contêiner.
    pub const CONTAINER_DISPATCH: &str = "container:dispatch";
    /// Ler contêineres.
    pub const CONTAINER_READ: &str = "container:read";
    /// Selar um contêiner.
    pub const CONTAINER_SEAL: &str = "container:seal";
    /// Ler o resumo de carga e telemetria.
    pub const CONTAINER_SUMMARY: &str = "container:summary";
    /// Alterar um contêiner.
    pub const CONTAINER_UPDATE: &str = "container:update";
    /// Embarcar carga.
    pub const MANIFEST_LOAD: &str = "manifest:load";
    /// Desembarcar carga.
    pub const MANIFEST_UNLOAD: &str = "manifest:unload";
    /// Ler o painel do pátio.
    pub const METRICS_READ: &str = "metrics:read";
    /// Listar as permissões registradas.
    pub const PERMISSION_LIST: &str = "permission:list";
    /// Cadastrar um produto.
    pub const PRODUCT_CREATE: &str = "product:create";
    /// Remover um produto.
    pub const PRODUCT_DELETE: &str = "product:delete";
    /// Ler produtos.
    pub const PRODUCT_READ: &str = "product:read";
    /// Alterar um produto.
    pub const PRODUCT_UPDATE: &str = "product:update";
    /// Criar um papel.
    pub const ROLE_CREATE: &str = "role:create";
    /// Ler papéis.
    pub const ROLE_LIST: &str = "role:list";
    /// Trocar as permissões de um papel.
    pub const ROLE_UPDATE_PERMISSIONS: &str = "role:update-permissions";
    /// Redefinir a senha de outro usuário.
    pub const USER_CHANGE_PASSWORD: &str = "user:change-password";
    /// Cadastrar um usuário.
    pub const USER_CREATE: &str = "user:create";
    /// Remover um usuário.
    pub const USER_DELETE: &str = "user:delete";
    /// Ler um usuário.
    pub const USER_GET: &str = "user:get";
    /// Listar usuários.
    pub const USER_LIST: &str = "user:list";
    /// Alterar um usuário.
    pub const USER_UPDATE: &str = "user:update";
    /// Trocar os papéis de um usuário.
    pub const USER_UPDATE_ROLES: &str = "user:update-roles";
}
