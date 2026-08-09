//! O repositório de marcadores, em memória.

use std::time::Duration;

use anyhow::{anyhow, bail};
use portmaster_domain::models::Marker;

use crate::cache::interno::marker_cache::MarkerCache;
use crate::repository::{MarkerGroupRepository, MarkerRepository};

/// O repositório de marcadores.
///
/// Genérico sobre o registro de grupos porque **valida** que o grupo existe
/// antes de gravar: sem isso, um erro de digitação no slug criaria um espaço de
/// nomes paralelo em silêncio, e nada do que fosse marcado nele seria encontrado
/// depois.
#[derive(Clone)]
pub struct MokaMarkerRepository<G> {
    /// Os marcadores, com o TTL que os expira sozinhos.
    cache: MarkerCache,
    /// Os grupos declarados, para recusar marcador de grupo que não existe.
    groups: G,
}

impl<G: MarkerGroupRepository> MokaMarkerRepository<G> {
    /// Monta o repositório sobre o cache do processo.
    pub(crate) const fn new(cache: MarkerCache, groups: G) -> Self {
        Self { cache, groups }
    }
}

impl<G: MarkerGroupRepository + Send + Sync> MarkerRepository for MokaMarkerRepository<G> {
    async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()> {
        if !self.groups.has(marker.group()).await? {
            return Err(anyhow!(
                "o grupo de marcador {:?} não foi registrado",
                marker.group()
            ));
        }

        let key = (marker.group().to_owned(), marker.key().to_owned());

        match (
            self.cache.0.get(&key).await.map(|(flag, _)| flag),
            marker.flag(),
        ) {
            (Some(true), true) => bail!("marcador já está válido: marcação reentrante"),
            (Some(false), true) => bail!("marcador foi invalidado e não pode ser revalidado"),
            _ => {}
        }

        self.cache
            .0
            .insert(key, (marker.flag(), Duration::from_secs(ttl_seconds)))
            .await;

        Ok(())
    }

    /// Inexistente, expirado e desligado respondem igual.
    ///
    /// Quem pergunta quer saber se pode seguir; distinguir os casos vazaria se
    /// um token já existiu.
    async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool> {
        Ok(self
            .cache
            .0
            .get(&(group.to_owned(), key.to_owned()))
            .await
            .is_some_and(|(flag, _)| flag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::interno::marker_group_cache::MarkerGroupCache;
    use crate::cache::interno::moka_marker_group_repository::MokaMarkerGroupRepository;
    use portmaster_domain::models::MarkerGroup;

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
    async fn repository() -> MokaMarkerRepository<MokaMarkerGroupRepository> {
        let metadata = MarkerGroupCache::new(100);
        let groups = MokaMarkerGroupRepository::new(metadata.clone());
        groups.register(&StubGroup("refresh-token")).await.unwrap();

        MokaMarkerRepository::new(
            MarkerCache::new(100),
            MokaMarkerGroupRepository::new(metadata),
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

    /// Um slug com erro de digitação criaria um espaço de nomes paralelo em
    /// que nada seria reencontrado.
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
}
