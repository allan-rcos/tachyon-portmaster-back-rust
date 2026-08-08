//! Snowflake — id de entidade, `i64` por dentro, base62 por fora.
//!
//! Escolhido para id de banco porque cabe num `BIGINT` de 8 bytes e ordena por
//! tempo: o índice primário cresce pela ponta em vez de fragmentar, o que um
//! UUID v4 não dá. O que o resto do sistema vê é sempre a string base62.
//!
//! ## Por que um `std::sync::Mutex`, que está banido
//!
//! O ban do `.clippy.toml` existe para impedir que estado de requisição vire
//! estado global sincronizado — o caminho certo para aquilo é o task-local.
//! Aqui é outra coisa: um recurso do **processo**, cuja seção crítica é o
//! incremento de um contador e não contém `.await` nenhum. É exatamente o caso
//! em que o mutex síncrono é o certo, e o assíncrono seria pior.

#[allow(
    clippy::disallowed_types,
    reason = "recurso de processo com seção crítica sem await — ver o doc de SnowflakeIdGenerator"
)]
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

use snowflake::SnowflakeIdGenerator as Generator;

use crate::id::{Base62, IntIdGenerator};

/// Era do gerador: 2024-01-01T00:00:00Z, a mesma do PHP.
///
/// É uma constante de build e não um segredo de runtime — dois deploys com
/// epochs diferentes emitiriam ids que se sobrepõem no tempo.
const EPOCH_MS: u64 = 1_704_067_200_000;

/// Gerador Snowflake compartilhado pelo processo.
///
/// O `Mutex` existe porque o algoritmo é sequencial por natureza: ele compara o
/// timestamp atual com o da última emissão e incrementa um contador quando os
/// dois caem no mesmo milissegundo. Duas threads gerando sem coordenação no
/// mesmo milissegundo produziriam o mesmo id.
#[allow(
    clippy::disallowed_types,
    reason = "gerador de ID Snowflake sequencial exige sincronização thread-safe entre requisições"
)]
pub struct SnowflakeIdGenerator {
    /// O gerador, atrás de um `Mutex`.
    ///
    /// Um Snowflake guarda o último instante e a sequência dentro dele para
    /// não repetir id no mesmo milissegundo, então há estado a proteger — e
    /// ele é o único do `domain`.
    inner: Mutex<Generator>,
}

impl SnowflakeIdGenerator {
    /// Monta o gerador para esta instância de deploy.
    ///
    /// `cluster_id` e `server_id` são o que distingue dois processos emitindo ao
    /// mesmo tempo; vêm dos segredos, não do build.
    pub(crate) fn new(cluster_id: i32, server_id: i32) -> Self {
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "EPOCH_MS é constante de build; UNIX_EPOCH + 2024 não chega perto de estourar SystemTime"
        )]
        let epoch = UNIX_EPOCH + Duration::from_millis(EPOCH_MS);
        #[allow(
            clippy::disallowed_types,
            reason = "recurso de processo com seção crítica sem await — ver o doc do tipo"
        )]
        Self {
            inner: Mutex::new(Generator::with_epoch(cluster_id, server_id, epoch)),
        }
    }

    /// Toma o gerador, ignorando o envenenamento do lock.
    ///
    /// Um lock envenenado significa que uma thread entrou em pânico segurando o
    /// gerador. O estado interno continua íntegro — é só um contador e um
    /// timestamp —, então seguir com ele é preferível a derrubar toda a emissão
    /// de id pelo resto da vida do processo.
    fn locked(&self) -> std::sync::MutexGuard<'_, Generator> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl IntIdGenerator for SnowflakeIdGenerator {
    fn next(&self) -> String {
        Base62::encode(self.locked().real_time_generate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// O caso que o `Mutex` existe para cobrir.
    ///
    /// Várias threads emitindo dentro do mesmo milissegundo.
    #[test]
    fn ids_sao_unicos_sob_concorrencia() {
        let generator = std::sync::Arc::new(SnowflakeIdGenerator::new(1, 1));
        let mut handles = Vec::new();

        for _ in 0..8 {
            let generator = generator.clone();
            handles.push(std::thread::spawn(move || {
                (0..500).map(|_| generator.next()).collect::<Vec<_>>()
            }));
        }

        let mut all = HashSet::new();
        for handle in handles {
            for id in handle.join().expect("thread de emissão entrou em pânico") {
                assert!(all.insert(id.clone()), "id repetido: {id}");
            }
        }

        assert_eq!(all.len(), 4_000);
    }

    #[test]
    fn ids_crescem_com_o_tempo() {
        let generator = SnowflakeIdGenerator::new(1, 1);
        let first = Base62::decode(&generator.next()).expect("id emitido deve decodificar");
        let second = Base62::decode(&generator.next()).expect("id emitido deve decodificar");
        assert!(second > first, "{second} deveria vir depois de {first}");
    }
}
