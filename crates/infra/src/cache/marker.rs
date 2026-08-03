//! Marcadores booleanos com prazo, sobre Moka.
//!
//! A `infra` nunca sabe o que uma marca significa. Aqui existe apenas "um
//! booleano com validade, num grupo conhecido" — que aquilo seja a sessão de
//! refresh de alguém é conhecimento do `api-http`.
//!
//! ## As regras de transição, e por que duas delas são erro
//!
//! | De | Para | |
//! |---|---|---|
//! | não existe | `true` | primeira marcação |
//! | `true` | `false` | invalidar, como num logout |
//! | `false` | `false` | idempotente |
//! | `false` | `true` | **erro** — não se ressuscita marca invalidada |
//! | `true` | `true` | **erro** — reentrância, pega submissão dupla |
//!
//! As duas recusas são de segurança. Reviver um `false` transformaria um logout
//! em algo reversível por quem tivesse o token antigo; remarcar um `true` é o
//! sinal de que a mesma marca chegou duas vezes, que é exatamente o que se quer
//! detectar num fluxo de idempotência.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail};
use moka::future::Cache;
use moka::policy::Expiry;

use portmaster_domain::marker::Marker;

use crate::repository::{MarkerGroupRepository, MarkerRepository};

/// Chave de um marcador: o grupo e o digest.
pub(crate) type MarkerKey = (String, String);

/// O que se guarda: o booleano e o prazo que ele pediu.
///
/// O prazo viaja junto do valor porque o Moka decide a expiração a partir da
/// entrada, não da chamada que a inseriu — e cada sessão vence no seu próprio
/// tempo, não num vencimento global.
pub(crate) type MarkerValue = (bool, Duration);

/// Cache de marcadores, com expiração por entrada.
pub(crate) type MarkerCache = Arc<Cache<MarkerKey, MarkerValue>>;

/// Ensina o Moka a ler o prazo que cada entrada carrega.
pub(crate) struct MarkerExpiry;

impl Expiry<MarkerKey, MarkerValue> for MarkerExpiry {
    fn expire_after_create(
        &self,
        _key: &MarkerKey,
        value: &MarkerValue,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(value.1)
    }

    fn expire_after_update(
        &self,
        _key: &MarkerKey,
        value: &MarkerValue,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        // Invalidar renova o prazo em vez de herdar o que restava do anterior:
        // a marca desligada precisa sobreviver ao menos tanto quanto o token que
        // ela recusa, senão expirar equivaleria a esquecer o logout.
        Some(value.1)
    }
}

/// Monta o cache de marcadores com a política de expiração por entrada.
pub(crate) fn build_cache(capacity: u64) -> MarkerCache {
    Arc::new(
        Cache::builder()
            .max_capacity(capacity)
            .expire_after(MarkerExpiry)
            .build(),
    )
}

/// O repositório de marcadores.
///
/// Genérico sobre o registro de grupos porque **valida** que o grupo existe
/// antes de gravar: sem isso, um erro de digitação no slug criaria um espaço de
/// nomes paralelo em silêncio, e nada do que fosse marcado nele seria encontrado
/// depois.
pub(crate) struct MokaMarkerRepository<G> {
    cache: MarkerCache,
    groups: G,
}

impl<G: MarkerGroupRepository> MokaMarkerRepository<G> {
    /// Monta o repositório sobre o cache do processo.
    pub(crate) fn new(cache: MarkerCache, groups: G) -> Self {
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
            self.cache.get(&key).await.map(|(flag, _)| flag),
            marker.flag(),
        ) {
            (Some(true), true) => bail!("marcador já está válido: marcação reentrante"),
            (Some(false), true) => bail!("marcador foi invalidado e não pode ser revalidado"),
            _ => {}
        }

        self.cache
            .insert(key, (marker.flag(), Duration::from_secs(ttl_seconds)))
            .await;

        Ok(())
    }

    async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool> {
        // Inexistente, expirado e desligado respondem igual. Quem pergunta quer
        // saber se pode seguir; distinguir os casos vazaria se um token já
        // existiu.
        Ok(self
            .cache
            .get(&(group.to_owned(), key.to_owned()))
            .await
            .is_some_and(|(flag, _)| flag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::metadata::{MarkerGroupCache, MokaMarkerGroupRepository};
    use portmaster_domain::metadata::marker_group::MarkerGroup;

    /// Marcador mínimo, sem passar pelo TableModule.
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

        MokaMarkerRepository::new(build_cache(100), MokaMarkerGroupRepository::new(metadata))
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

    #[tokio::test]
    async fn nao_revalida_marca_invalidada() {
        // Se isto passasse, um logout seria reversível por quem guardasse o
        // token antigo.
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

    #[tokio::test]
    async fn recusa_grupo_nao_registrado() {
        // Um slug com erro de digitação criaria um espaço de nomes paralelo em
        // que nada seria reencontrado.
        let repository = repository().await;
        let stray = StubMarker {
            group: "refresh-tokens",
            key: "abc",
            flag: true,
        };

        assert!(repository.put(&stray, 60).await.is_err());
    }
}
