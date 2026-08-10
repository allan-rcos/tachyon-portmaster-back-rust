//! A tradução entre o id da coluna e o id do fio.
//!
//! Esta é a única camada que vê o `i64` do Snowflake. Ao ler uma linha, deriva o
//! base62 que o trait expõe; ao gravar, decodifica de volta. Se o id físico
//! mudar um dia, só este arquivo muda.
//!
//! A validação de índice de enum saiu daqui: ela é do ponto de conversão, e
//! cada `FromRow` e cada leitura de DQL a faz com o `ok_or_else` de quem sabe
//! que enum está lendo.

use anyhow::Context;

/// Converte entre o id da coluna e o id do fio.
///
/// Namespace no molde do `Base62`: as duas funções são a mesma fronteira vista
/// dos dois lados, e nenhuma faz sentido fora dela.
pub(crate) struct Codec;

impl Codec {
    /// Traduz um id base62 no `BIGINT` da coluna.
    ///
    /// Um id inválido aqui não é corrupção interna: é uma URL inventada, e precisa
    /// falhar como entrada malformada em vez de virar uma consulta por um id
    /// arbitrário. O `try_from` cobre o caso extra que a decodificação não cobre:
    /// uma string curta o bastante para caber num `u128` e grande demais para
    /// caber na coluna.
    pub fn decode_id(id: &str) -> anyhow::Result<i64> {
        let decoded =
            base62::decode(id).with_context(|| format!("id fora do formato base62: {id}"))?;

        i64::try_from(decoded).with_context(|| format!("id base62 fora da faixa da coluna: {id}"))
    }

    /// Traduz um `BIGINT` no id base62 que o trait expõe.
    ///
    /// Id negativo não existe: o Snowflake não acende o bit de sinal. Se um
    /// aparecer, ele veio de uma linha que o schema não deveria admitir, e
    /// codificá-lo como zero é preferível a um pânico de conversão no meio de
    /// uma leitura.
    pub fn encode_id(id: i64) -> String {
        base62::encode(u64::try_from(id).unwrap_or_default())
    }
}
