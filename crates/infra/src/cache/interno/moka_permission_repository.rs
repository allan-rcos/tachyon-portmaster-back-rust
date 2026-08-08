//! O repositório de permissões, em memória.

use portmaster_domain::models::Permission;

use crate::cache::interno::permission_cache::PermissionCache;
use crate::repository::PermissionRepository;

/// Registro de permissões.
pub struct MokaPermissionRepository {
    cache: PermissionCache,
}

impl MokaPermissionRepository {
    /// Monta o registro sobre o mapa do processo.
    pub(crate) const fn new(cache: PermissionCache) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::interno::marker_group_cache::MarkerGroupCache;
    use crate::cache::interno::moka_marker_group_repository::MokaMarkerGroupRepository;
    use crate::repository::MarkerGroupRepository as _;
    use portmaster_domain::models::MarkerGroup;
    use pretty_assertions::assert_eq;

    fn cache() -> PermissionCache {
        PermissionCache::new(100)
    }

    /// Permissão mínima, sem passar pelo `TableModule`.
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

    /// Grupo mínimo, sem passar pelo `TableModule`.
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
