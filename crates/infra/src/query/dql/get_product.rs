//! A consulta de um produto pelo id.

use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::dql::list_products::read_item;
use crate::query::views::ProductViewItem;
use crate::query::{Dql, SqlDql};

/// Um produto pelo id.
///
/// Falível porque um id em base62 pode simplesmente não ser base62 — uma URL
/// inventada. Recusar aqui é melhor do que abrir transação e consultar por um
/// número arbitrário.
pub fn get_product(id: &str) -> anyhow::Result<impl SqlDql<View = Option<ProductViewItem>>> {
    Ok(GetProduct {
        id: Codec::decode_id(id)?,
    })
}

/// Um produto pelo id.
///
/// Repete o filtro de soft-delete da escrita: sem ele, um produto removido
/// reapareceria na leitura.
struct GetProduct {
    /// O alvo, como `BIGINT`.
    id: i64,
}

impl Dql for GetProduct {
    type View = Option<ProductViewItem>;

    fn cache_key(&self) -> String {
        format!("get_product:{}", self.id)
    }
}

impl SqlDql for GetProduct {
    /// As colunas são nomeadas em vez de `*`: a projeção é o contrato da
    /// hidratação, e um `SELECT *` faria uma coluna nova entrar na consulta sem
    /// que ninguém a pedisse.
    fn build(&self) -> (String, Params) {
        let sql = "SELECT p.id, p.name, p.density, p.risk_class FROM products p \
                   WHERE p.id = :id AND p.deleted_at IS NULL LIMIT 1";

        (
            sql.to_owned(),
            vec![("id".to_owned(), Value::Int(self.id))].into(),
        )
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        rows.first().map(read_item).transpose()
    }
}
