//! A tradução entre o que a coluna guarda e o que o domínio expõe.
//!
//! Esta é a única camada que vê o `i64` do Snowflake e o índice numérico de um
//! enum. Ao ler uma linha, deriva o base62 e a variante que o trait expõe; ao
//! gravar, decodifica de volta. Se o id físico mudar um dia, só este arquivo
//! muda.

use anyhow::{anyhow, Context};
use portmaster_domain::id::Base62;

/// Converte entre a representação da coluna e a do domínio.
///
/// Namespace, no molde do `Base62`: as três funções são a mesma fronteira vista
/// de três ângulos, e nenhuma faz sentido fora dela.
pub(crate) struct Codec;

impl Codec {
    /// Traduz um id base62 no `BIGINT` da coluna.
    ///
    /// Um id inválido aqui não é corrupção interna: é uma URL inventada, e precisa
    /// falhar como entrada malformada em vez de virar uma consulta por um id
    /// arbitrário.
    pub fn decode_id(id: &str) -> anyhow::Result<i64> {
        Base62::decode(id).with_context(|| format!("id fora do formato base62: {id}"))
    }

    /// Traduz um `BIGINT` no id base62 que o trait expõe.
    pub fn encode_id(id: i64) -> String {
        Base62::encode(id)
    }

    /// Traduz o índice numérico de uma coluna no enum de domínio.
    ///
    /// Um valor que não corresponde a variante nenhuma é uma linha que o schema não
    /// deveria admitir — falhar alto aqui é melhor do que escolher uma variante por
    /// aproximação e seguir com o dado errado.
    pub fn decode_enum<T>(
        value: i32,
        from: impl Fn(i32) -> Option<T>,
        name: &str,
    ) -> anyhow::Result<T> {
        from(value)
            .ok_or_else(|| anyhow!("valor {value} não corresponde a nenhuma variante de {name}"))
    }
}
