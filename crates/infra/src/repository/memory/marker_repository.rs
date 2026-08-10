//! Persistência de marcadores em memória.

use std::time::Duration;

use anyhow::{anyhow, bail};
use portmaster_domain::domain::Marker;

use crate::repository::{MarkerGroupRepository, MarkerRepository};
use crate::scope::memory::memory_store::MemoryStore;

/// A marca ligada, como ela é gravada.
const ON: u8 = 1;

/// O repositório de marcadores.
///
/// Genérico sobre o registro de grupos porque **valida** que o grupo existe
/// antes de gravar: sem isso, um erro de digitação no slug criaria um espaço de
/// nomes paralelo em silêncio, e nada do que fosse marcado nele seria encontrado
/// depois.
///
/// Aqui o grupo do store é o grupo do próprio marcador, e não uma `const`: um
/// marcador já nasce dizendo a que namespace pertence.
#[derive(Clone)]
pub struct MarkerMemoryRepository<S, G> {
    /// De onde a memória da tarefa vem.
    store: S,
    /// Os grupos declarados, para recusar marcador de grupo que não existe.
    groups: G,
}

impl<S, G> MarkerMemoryRepository<S, G> {
    /// Monta o repositório.
    pub(crate) const fn new(store: S, groups: G) -> Self {
        Self { store, groups }
    }
}

impl<S: MemoryStore + Send + Sync, G: MarkerGroupRepository + Send + Sync> MarkerRepository
    for MarkerMemoryRepository<S, G>
{
    async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()> {
        if !self.groups.has(marker.group()).await? {
            return Err(anyhow!(
                "o grupo de marcador {:?} não foi registrado",
                marker.group()
            ));
        }

        let current = self.store.get(marker.group(), marker.key()).await?;

        match (
            current.as_deref().map(|bytes| bytes.first() == Some(&ON)),
            marker.flag(),
        ) {
            (Some(true), true) => bail!("marcador já está válido: marcação reentrante"),
            (Some(false), true) => bail!("marcador foi invalidado e não pode ser revalidado"),
            _ => {}
        }

        self.store
            .put(
                marker.group(),
                marker.key(),
                vec![u8::from(marker.flag())],
                Some(Duration::from_secs(ttl_seconds)),
            )
            .await
    }

    /// Inexistente, expirado e desligado respondem igual.
    ///
    /// Quem pergunta quer saber se pode seguir; distinguir os casos vazaria se
    /// um token já existiu.
    async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool> {
        Ok(self
            .store
            .get(group, key)
            .await?
            .is_some_and(|bytes| bytes.first() == Some(&ON)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::memory::marker_group_repository::MarkerGroupMemoryRepository;
    use crate::scope::memory::intern::moka_cache::MokaCache;
    use crate::scope::memory::intern::moka_store::MokaStore;
    use portmaster_domain::domain::MarkerGroup;

    /// Marcador mínimo, sem passar pelo `TableModule`.
    struct StubMarker {
        group: &'static str,
        key: &'static str,
        flag: bool,
    }

    impl Marker for StubMarker {
        fn group(&self) -> &str {
            self.group
        }
        fn key(&self) -> &str {
            self.key
        }
        fn flag(&self) -> bool {
            self.flag
        }
    }

    struct StubGroup(&'static str);
    impl MarkerGroup for StubGroup {
        fn slug(&self) -> &str {
            self.0
        }
    }

    /// Repositório com o grupo `refresh-token` já registrado.
    async fn repository(
    ) -> MarkerMemoryRepository<MokaStore, MarkerGroupMemoryRepository<MokaStore>> {
        let metadata = MokaStore::new("marker-group", MokaCache::new(100));
        let groups = MarkerGroupMemoryRepository::new(metadata.clone());
        groups.register(&StubGroup("refresh-token")).await.unwrap();

        MarkerMemoryRepository::new(
            MokaStore::new("marker", MokaCache::new(100)),
            MarkerGroupMemoryRepository::new(metadata),
        )
    }

    fn marker(flag: bool) -> StubMarker {
        StubMarker {
            group: "refresh-token",
            key: "abc123",
            flag,
        }
    }

    #[tokio::test]
    async fn marcar_e_consultar() {
        let repository = repository().await;
        repository.put(&marker(true), 60).await.unwrap();

        assert!(repository
            .is_valid("refresh-token", "abc123")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn marca_inexistente_nao_e_valida() {
        let repository = repository().await;
        assert!(!repository
            .is_valid("refresh-token", "nunca-visto")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn invalidar_e_permitido_e_idempotente() {
        // O caminho do logout, e do logout chamado duas vezes.
        let repository = repository().await;
        repository.put(&marker(true), 60).await.unwrap();
        repository.put(&marker(false), 60).await.unwrap();
        repository.put(&marker(false), 60).await.unwrap();

        assert!(!repository
            .is_valid("refresh-token", "abc123")
            .await
            .unwrap());
    }

    /// Se isto passasse, um logout seria reversível por quem guardasse o token
    /// antigo.
    #[tokio::test]
    async fn nao_revalida_marca_invalidada() {
        let repository = repository().await;
        repository.put(&marker(true), 60).await.unwrap();
        repository.put(&marker(false), 60).await.unwrap();

        assert!(repository.put(&marker(true), 60).await.is_err());
    }

    #[tokio::test]
    async fn nao_remarca_o_que_ja_esta_valido() {
        let repository = repository().await;
        repository.put(&marker(true), 60).await.unwrap();

        assert!(repository.put(&marker(true), 60).await.is_err());
    }

    /// Um slug com erro de digitação criaria um espaço de nomes paralelo em que
    /// nada seria reencontrado.
    #[tokio::test]
    async fn recusa_grupo_nao_registrado() {
        let repository = repository().await;
        let stray = StubMarker {
            group: "refresh-tokens",
            key: "abc",
            flag: true,
        };

        assert!(repository.put(&stray, 60).await.is_err());
    }

    /// O marcador e o grupo vivem em stores distintos: se dividissem o mesmo,
    /// um grupo declarado responderia como marcador válido.
    #[tokio::test]
    async fn grupo_declarado_nao_e_marcador_valido() {
        let repository = repository().await;

        assert!(!repository
            .is_valid("marker-group", "refresh-token")
            .await
            .unwrap());
    }
}
