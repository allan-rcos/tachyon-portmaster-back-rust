//! Registro de metadados de sistema, sobre cache.
//!
//! Apesar do módulo estar sob `mariadb`, **nada aqui toca o banco**: permissões e
//! grupos de marcador são preenchidos em código no boot, são imutáveis depois
//! disso e são lidos a cada verificação de autorização. Persistir isso numa
//! tabela seria consulta cara para um dado que já se conhece.
//!
//! O modelo anterior precisava de uma tabela `MEMORY` porque rodava em quatro
//! processos forkados, cada um com a sua memória — o banco era o único terreno
//! comum. Aqui é um processo só, e o `Arc<Cache>` já é compartilhado por todas as
//! threads.

use std::sync::Arc;

use moka::future::Cache;
use portmaster_domain::metadata::marker_group::MarkerGroup;
use portmaster_domain::metadata::permission::Permission;

use crate::repository::{MarkerGroupRepository, PermissionRepository};

/// O mapa de um registro.
///
/// O valor é `()`: o que importa é a presença da chave. Sem TTL nem teto de
/// despejo por tempo — um metadado despejado seria uma permissão que some do
/// catálogo com o processo ainda de pé.
type Slugs = Arc<Cache<String, ()>>;

/// O mapa das permissões.
///
/// É um tipo próprio, e não um `Arc<Cache<..>>` solto, porque os dois registros
/// **não podem** dividir o mesmo mapa: `PermissionRepository::all()` alimenta o
/// papel que o `POST /setup` cria, e um grupo de marcador caído ali vira uma
/// concessão que ninguém declarou. Tipos distintos transformam essa confusão num
/// erro de compilação em vez de numa permissão a mais no administrador.
#[derive(Clone)]
pub(crate) struct PermissionCache(Slugs);

impl PermissionCache {
    /// Monta o mapa com a capacidade dada.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(Cache::new(capacity)))
    }
}

/// O mapa dos grupos de marcador. Ver [`PermissionCache`].
#[derive(Clone)]
pub(crate) struct MarkerGroupCache(Slugs);

impl MarkerGroupCache {
    /// Monta o mapa com a capacidade dada.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(Cache::new(capacity)))
    }
}

/// Registro de permissões.
pub(crate) struct MokaPermissionRepository {
    cache: PermissionCache,
}

impl MokaPermissionRepository {
    /// Monta o registro sobre o mapa do processo.
    pub(crate) fn new(cache: PermissionCache) -> Self {
        Self { cache }
    }
}

impl PermissionRepository for MokaPermissionRepository {
    async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()> {
        // Idempotente por slug: cada caso de uso declara a sua permissão ao ser
        // construído, e nada garante que isso aconteça uma vez só.
        self.cache.0.insert(permission.slug().to_owned(), ()).await;
        Ok(())
    }

    async fn all(&self) -> anyhow::Result<Vec<String>> {
        let mut slugs: Vec<String> = self
            .cache
            .0
            .iter()
            .map(|(slug, ())| slug.as_ref().clone())
            .collect();

        // Ordenado porque a iteração do Moka não tem ordem definida, e uma
        // listagem que muda de ordem a cada chamada é ruim de ler e pior de
        // testar.
        slugs.sort_unstable();
        Ok(slugs)
    }

    async fn has(&self, slug: &str) -> anyhow::Result<bool> {
        Ok(self.cache.0.contains_key(slug))
    }
}

/// Registro de grupos de marcador.
pub(crate) struct MokaMarkerGroupRepository {
    cache: MarkerGroupCache,
}

impl MokaMarkerGroupRepository {
    /// Monta o registro sobre o mapa do processo.
    pub(crate) fn new(cache: MarkerGroupCache) -> Self {
        Self { cache }
    }
}

impl MarkerGroupRepository for MokaMarkerGroupRepository {
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()> {
        self.cache.0.insert(group.slug().to_owned(), ()).await;
        Ok(())
    }

    async fn has(&self, slug: &str) -> anyhow::Result<bool> {
        Ok(self.cache.0.contains_key(slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn cache() -> PermissionCache {
        PermissionCache::new(100)
    }

    /// Permissão mínima, sem passar pelo TableModule.
    struct StubPermission(&'static str);
    impl Permission for StubPermission {
        fn slug(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn registrar_a_mesma_permissao_duas_vezes_nao_duplica() {
        // Cada caso de uso declara a sua permissão ao ser construído, e o
        // provider pode construí-lo mais de uma vez.
        let repository = MokaPermissionRepository::new(cache());

        repository
            .register(&StubPermission("product:create"))
            .await
            .unwrap();
        repository
            .register(&StubPermission("product:create"))
            .await
            .unwrap();

        assert_eq!(repository.all().await.unwrap(), vec!["product:create"]);
    }

    #[tokio::test]
    async fn a_listagem_sai_ordenada() {
        let repository = MokaPermissionRepository::new(cache());

        for slug in ["user:list", "container:seal", "product:create"] {
            repository.register(&StubPermission(slug)).await.unwrap();
        }

        assert_eq!(
            repository.all().await.unwrap(),
            vec!["container:seal", "product:create", "user:list"]
        );
    }

    #[tokio::test]
    async fn has_responde_pelo_que_foi_registrado() {
        let repository = MokaPermissionRepository::new(cache());
        repository
            .register(&StubPermission("metrics:read"))
            .await
            .unwrap();

        assert!(repository.has("metrics:read").await.unwrap());
        assert!(!repository.has("metrics:write").await.unwrap());
    }

    /// Grupo mínimo, sem passar pelo TableModule.
    struct StubGroup(&'static str);
    impl MarkerGroup for StubGroup {
        fn slug(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn um_grupo_de_marcador_nao_entra_no_catalogo_de_permissoes() {
        // Os dois registros dividiam um mapa só, e o grupo `refresh-token`
        // aparecia na listagem de permissões — e era concedido ao papel que o
        // `POST /setup` cria, que recebe tudo que estiver registrado.
        let permissions = MokaPermissionRepository::new(PermissionCache::new(100));
        let groups = MokaMarkerGroupRepository::new(MarkerGroupCache::new(100));

        groups.register(&StubGroup("refresh-token")).await.unwrap();

        assert!(permissions.all().await.unwrap().is_empty());
        assert!(!permissions.has("refresh-token").await.unwrap());
        assert!(groups.has("refresh-token").await.unwrap());
    }

    #[tokio::test]
    async fn o_cache_e_compartilhado_entre_threads() {
        // A garantia que substituiu a tabela MEMORY: um processo, muitas
        // threads, um mapa só.
        let shared = cache();
        let writer = MokaPermissionRepository::new(shared.clone());
        let reader = MokaPermissionRepository::new(shared);

        tokio::spawn(async move {
            writer
                .register(&StubPermission("user:create"))
                .await
                .unwrap();
        })
        .await
        .unwrap();

        assert!(reader.has("user:create").await.unwrap());
    }
}
