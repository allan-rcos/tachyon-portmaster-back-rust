//! Metadados de sistema.

use crate::commands::metadata::RegisterPermissionCommand;
use crate::error::MetadataError;
use crate::queries::metadata::ListPermissionsQuery;

/// O que a apresentação pode pedir sobre metadados.
#[trait_variant::make(Send)]
pub trait MetadataService {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Sem o argumento `registrar` que os outros serviços recebem: este **é** o
    /// registrador, e pedir a si mesmo por parâmetro só acrescentaria cerimônia.
    /// O slug continua sendo `const` privada da implementação.
    async fn declare_permissions(&self) -> Result<(), MetadataError>;

    /// Registra uma permissão no catálogo.
    ///
    /// Chamada no boot, uma vez por slug de cada serviço. Precisa acontecer
    /// antes da primeira requisição: o `POST /setup` concede ao primeiro papel
    /// tudo que estiver registrado, e um catálogo incompleto daria 403 em
    /// endpoints que deveriam funcionar — com a causa invisível, porque o papel
    /// do administrador simplesmente não teria a permissão.
    async fn register_permission(
        &self,
        command: RegisterPermissionCommand,
    ) -> Result<(), MetadataError>;

    /// Os slugs registrados, em ordem.
    async fn list_permissions(
        &self,
        query: ListPermissionsQuery,
    ) -> Result<Vec<String>, MetadataError>;
}
