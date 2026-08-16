//! A consulta de um produto pelo id.

use sqlx::mysql::{MySql, MySqlRow};
use sqlx::QueryBuilder;

use crate::entity::codec::Codec;
use crate::query::dql::list_products::read_item;
use crate::query::views::ProductViewItem;
use crate::query::{Dql, SqlDql};

/// As colunas que a View de produto precisa.
const COLUMNS: &str = "p.id, p.name, p.density, p.risk_class";

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
    fn build(&self) -> QueryBuilder<MySql> {
        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);
        builder.push(" FROM products p WHERE p.id = ");
        builder.push_bind(self.id);
        builder.push(" AND p.deleted_at IS NULL LIMIT 1");

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(read_item).transpose()
    }
}
