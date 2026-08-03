//! As entities: onde o domínio encosta na tabela.
//!
//! Cada entity implementa o trait de domínio correspondente **e** concentra tudo
//! que é de banco — nome de tabela, nome de coluna, tipo de cada campo. Isso
//! mantém o model do `domain` limpo de qualquer vestígio de persistência, e é o
//! que permitiria trocar de banco mexendo só aqui.
//!
//! Como o trait de domínio é somente-leitura, uma entity **não consegue editar**
//! um objeto vindo de outra camada: ela o **recria** a partir do trait, por um
//! `from_domain`. A ausência de setters não é inconveniente, é a garantia.
//!
//! ## O `i64` para de existir aqui
//!
//! Esta é a única camada que vê o inteiro do Snowflake. Ao ler uma linha, deriva
//! o base62 que o trait expõe; ao gravar, decodifica o base62 de volta para
//! `BIGINT`. Se o id físico mudar um dia, só este mapeamento muda.

pub(crate) mod container;
pub(crate) mod manifest;
pub(crate) mod product;
pub(crate) mod role;
pub(crate) mod user;

use anyhow::{anyhow, Context};
use portmaster_domain::id::base62;

/// Traduz um id base62 no `BIGINT` da coluna.
///
/// Um id inválido aqui não é corrupção interna: é uma URL inventada, e precisa
/// falhar como entrada malformada em vez de virar uma consulta por um id
/// arbitrário.
pub(crate) fn decode_id(id: &str) -> anyhow::Result<i64> {
    base62::decode(id).with_context(|| format!("id fora do formato base62: {id}"))
}

/// Traduz um `BIGINT` no id base62 que o trait expõe.
pub(crate) fn encode_id(id: i64) -> String {
    base62::encode(id)
}

/// Traduz o índice numérico de uma coluna no enum de domínio.
///
/// Um valor que não corresponde a variante nenhuma é uma linha que o schema não
/// deveria admitir — falhar alto aqui é melhor do que escolher uma variante por
/// aproximação e seguir com o dado errado.
pub(crate) fn decode_enum<T>(
    value: i32,
    from: impl Fn(i32) -> Option<T>,
    name: &str,
) -> anyhow::Result<T> {
    from(value).ok_or_else(|| anyhow!("valor {value} não corresponde a nenhuma variante de {name}"))
}
