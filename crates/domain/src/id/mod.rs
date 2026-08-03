//! Geração do id de uma entidade persistida.
//!
//! O id nasce **aqui**, no TableModule, porque escolher a identidade de uma
//! entidade é regra de negócio — não é o repositório que decide quem ela é.
//!
//! O trait é `pub(crate)`: um gerador de id nas mãos do `app` permitiria montar
//! uma entidade sem passar pelo TableModule, que é justamente onde a validação
//! mora. Quem precisa de um id o obtém construindo o objeto pela regra.
//!
//! Os outros dois geradores do sistema não são daqui: o NanoID do refresh token e
//! o xid do `request_id` não são identidade de entidade e vivem na `infra`.
//!
//! A **estratégia** é escolhida por feature de compilação — decisão de
//! arquitetura, não um `if` de runtime. Os **parâmetros de identidade**
//! (`cluster_id`/`server_id`) são de deploy e chegam por segredo.

pub mod base62;

#[cfg(feature = "id-snowflake")]
pub(crate) mod snowflake;

/// Gera o id de uma entidade persistida, já compactado em base62.
///
/// O `&self` não é um detalhe: o gerador é compartilhado por todas as threads do
/// processo, então a impl guarda o seu estado atrás de um lock. É diferente do
/// modelo de processos forkados, em que cada worker tinha o seu próprio contador
/// e a unicidade dependia de nunca repetir o par cluster/server.
pub(crate) trait IntIdGenerator {
    /// Um id novo, único e crescente no tempo.
    fn next(&self) -> String;
}
