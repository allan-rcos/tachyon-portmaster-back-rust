//! A leitura de uma coluna de uma linha de resultado.

use anyhow::Context as _;
use mysql_async::prelude::FromValue;
use mysql_async::Row;

/// Lê uma coluna pelo nome, com o erro já explicado.
///
/// Namespace no molde do [`Codec`](crate::entity::codec::Codec): a hidratação de
/// uma View lê dezenas de colunas, e sem isto cada leitura repetiria as duas
/// mesmas frases de erro.
///
/// Duas falhas diferentes, ditas em separado de propósito: a coluna **não veio**
/// na consulta — o `SELECT` não a projetou, e é erro de quem escreveu a consulta
/// — ou veio e **não converte**, que é linha em desacordo com o schema. O valor
/// recusado aparece no encadeamento, porque é o que o erro do driver carrega.
pub(crate) struct Column;

impl Column {
    /// O valor da coluna, no tipo que a View quer.
    pub fn of<T: FromValue>(row: &Row, name: &str) -> anyhow::Result<T> {
        row.get_opt::<T, _>(name)
            .with_context(|| format!("a consulta não projetou a coluna `{name}`"))?
            .with_context(|| format!("a coluna `{name}` não veio no tipo que a View espera"))
    }
}
