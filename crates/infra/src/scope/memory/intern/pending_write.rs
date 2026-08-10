//! Uma escrita que a tarefa fez e ainda não publicou.

use anyhow::anyhow;

use std::sync::Arc;
use std::time::Duration;

use crate::scope::memory::intern::moka_cache::MokaCache;

/// O que fazer com o grupo quando a tarefa confirmar.
pub(super) enum Operation {
    /// Gravar a entrada, com o prazo dela.
    Put {
        /// A entrada dentro do grupo.
        key: String,
        /// Os bytes gravados.
        value: Arc<Vec<u8>>,
        /// Quanto tempo ela vale; `None` nunca vence.
        ttl: Option<Duration>,
    },
    /// Descartar o grupo inteiro.
    ClearGroup,
}

/// Uma escrita enfileirada, com o cache que vai recebê-la.
///
/// O cache viaja **junto** da operação, e não é buscado num global na hora de
/// publicar: quem enfileira é o repositório, e o repositório já tem o handle
/// injetado. É isso que permite à camada de escopo não conhecer recurso nenhum.
pub(super) struct PendingWrite {
    /// Onde a escrita será publicada.
    pub(super) cache: MokaCache,
    /// De qual store ela saiu, para que a leitura não confunda dois recortes.
    pub(super) store: &'static str,
    /// O namespace dentro do store.
    pub(super) group: String,
    /// O que fazer.
    pub(super) operation: Operation,
}

impl PendingWrite {
    /// Publica a escrita no cache.
    pub(super) async fn apply(self) -> anyhow::Result<()> {
        match self.operation {
            Operation::Put { key, value, ttl } => {
                self.cache.0.insert((self.group, key), (value, ttl)).await;
            }
            Operation::ClearGroup => {
                let group = self.group;
                self.cache
                    .0
                    .invalidate_entries_if(move |(entry_group, _), _| *entry_group == group)
                    .map_err(|error| anyhow!("falha ao descartar o grupo em memória: {error}"))?;
            }
        }

        Ok(())
    }
}
