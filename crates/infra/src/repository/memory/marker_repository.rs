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
#[path = "tests/marker_repository_test.rs"]
mod tests;
