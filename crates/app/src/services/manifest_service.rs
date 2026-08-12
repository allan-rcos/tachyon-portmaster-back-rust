//! Carga e telemetria.

use crate::commands::manifest::MoveItemCommand;
use crate::error::ManifestError;
use crate::services::MetadataService;
use portmaster_domain::domain::Container;

/// O que a apresentação pode pedir sobre manifesto.
///
/// Os dois devolvem o **contêiner** no estado novo, e não a linha movimentada:
/// quem embarca quer saber quanto o contêiner passou a pesar e se saiu de vazio,
/// que é o que decide o próximo movimento. A linha em si o chamador já tem — foi
/// ele quem a pediu.
#[trait_variant::make(Send)]
pub trait ManifestService {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ManifestError>;

    /// Embarca carga num contêiner, e devolve o contêiner resultante.
    async fn load(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, ManifestError>;

    /// Desembarca carga de um contêiner, e devolve o contêiner resultante.
    async fn unload(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, ManifestError>;
}
