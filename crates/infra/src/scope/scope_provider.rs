//! Quem serve o escopo da tarefa, dos dois lados.

use crate::config::InfraSecrets;
use crate::scope::database::DatabaseScopeProvider;
use crate::scope::database::MySqlTransaction;
use crate::scope::memory::MemoryScopeProvider;
use crate::scope::memory::MemoryStore;

/// O escopo da tarefa, banco e memória.
///
/// Encapsula os dois providers de baixo: quem constrói um repositório pede aqui
/// e não sabe se o que recebe fala com o `MariaDB` ou com um mapa em memória.
///
/// A unidade de trabalho **não** sai por aqui para o `app`: quem a entrega é o
/// `MasterScope::run`, a partir das layers registradas no linker. O que este
/// provider serve é o handle que um repositório carrega para alcançar a
/// transação da tarefa — coisa diferente.
pub(crate) struct ScopeProvider;

impl ScopeProvider {
    /// Abre o pool com estes segredos, e larga o que estava aberto.
    pub(crate) fn install_database(secrets: &InfraSecrets) -> anyhow::Result<()> {
        DatabaseScopeProvider::install(secrets)
    }

    /// O acesso ao banco.
    pub(crate) fn database(
    ) -> anyhow::Result<impl MySqlTransaction + Sync + Clone + use<> + 'static> {
        DatabaseScopeProvider::unit_of_work()
    }

    /// Confirma que o banco responde.
    pub(crate) async fn ping() -> anyhow::Result<()> {
        DatabaseScopeProvider::ping().await
    }

    /// O recorte das permissões.
    pub(crate) fn permissions() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        MemoryScopeProvider::permissions()
    }

    /// O recorte dos grupos de marcador.
    pub(crate) fn marker_groups() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        MemoryScopeProvider::marker_groups()
    }

    /// O recorte dos marcadores.
    pub(crate) fn markers() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        MemoryScopeProvider::markers()
    }

    /// O recorte do cache de leitura.
    pub(crate) fn views() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        MemoryScopeProvider::views()
    }
}
