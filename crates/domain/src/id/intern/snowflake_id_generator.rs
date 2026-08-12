//! Snowflake — id de entidade, inteiro por dentro, base62 por fora.
//!
//! Escolhido para id de banco porque cabe num `BIGINT` de 8 bytes e ordena por
//! tempo: o índice primário cresce pela ponta em vez de fragmentar, o que um
//! UUID v4 não dá. O que o resto do sistema vê é sempre a string base62.
//!
//! O gerador é do `snowflaked`, na variante `sync`: o estado — o último instante
//! e a sequência dentro dele — mora num `AtomicU64`, e `generate` toma `&self`.
//! É o que permite este arquivo ser só o que ele deveria ser, um adaptador de
//! quinze linhas entre uma lib pronta e o contrato do domínio.

use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use snowflaked::sync::Generator;

use crate::id::DatabaseIdGenerator;

/// Era do gerador: 2024-01-01T00:00:00Z, a mesma do PHP.
///
/// É uma constante de build e não um segredo de runtime — dois deploys com
/// epochs diferentes emitiriam ids que se sobrepõem no tempo.
const EPOCH_MS: u64 = 1_704_067_200_000;

/// Quantos servidores cabem num cluster, e a largura do campo de cada um.
const NODE_COUNT: u16 = 32;

/// O maior valor que cluster e servidor admitem.
const NODE_MAX: u16 = 31;

/// Gerador Snowflake compartilhado pelo processo.
///
/// O `Arc` é o que faz um clone deste gerador continuar sendo **o mesmo**
/// gerador, e é a exceção de borda que a DI estática admite pela razão mais
/// literal possível: dois geradores independentes com o mesmo
/// `cluster_id`/`server_id` emitiriam ids repetidos. Ele não pode existir em
/// mais de um lugar ao mesmo tempo, então é compartilhado em vez de copiado.
#[derive(Clone)]
pub(crate) struct SnowflakeIdGenerator {
    /// O gerador da lib, compartilhado por todas as threads do processo.
    inner: Arc<Generator>,
}

impl SnowflakeIdGenerator {
    /// Monta o gerador para esta instância de deploy.
    ///
    /// `cluster_id` e `server_id` são o que distingue dois processos emitindo ao
    /// mesmo tempo; vêm dos segredos, não do build.
    pub(crate) fn new(cluster_id: i32, server_id: i32) -> Self {
        let epoch = UNIX_EPOCH
            .checked_add(Duration::from_millis(EPOCH_MS))
            .unwrap_or(UNIX_EPOCH);

        Self {
            inner: Arc::new(
                Generator::builder()
                    .instance(instance_of(cluster_id, server_id))
                    .epoch(epoch)
                    .build(),
            ),
        }
    }
}

impl DatabaseIdGenerator for SnowflakeIdGenerator {
    fn next(&self) -> String {
        base62::encode(self.inner.generate::<u64>())
    }
}

/// Junta cluster e servidor nos dez bits de `instance` do Snowflake.
///
/// Cada metade vai de 0 a 31, o que ocupa cinco bits; juntas cabem exatamente
/// nos dez que o algoritmo reserva para identificar quem emitiu. Valor fora da
/// faixa é preso na borda: o construtor do `snowflaked` entra em pânico acima do
/// máximo, e derrubar o boot por um segredo mal preenchido troca um problema de
/// configuração por um processo que não sobe.
fn instance_of(cluster_id: i32, server_id: i32) -> u16 {
    let cluster = u16::try_from(cluster_id).unwrap_or_default().min(NODE_MAX);
    let server = u16::try_from(server_id).unwrap_or_default().min(NODE_MAX);

    cluster.saturating_mul(NODE_COUNT).saturating_add(server)
}

#[cfg(test)]
#[path = "tests/snowflake_id_generator_test.rs"]
mod tests;
