//! A consulta de um produto pelo id.

use crate::query::dql::product_reader::ProductReader;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::ProductViewItem;
use crate::query::{Dql, SqlDql};
use sqlx::mysql::MySqlRow;

/// As colunas que a View de produto precisa.
///
/// Nomeadas em vez de `*`: a projeção é o contrato da hidratação, e um `SELECT *`
/// faria uma coluna nova entrar na consulta sem que ninguém a pedisse.
const COLUMNS: &str = "p.id, p.name, p.density, p.risk_class";

/// Um produto pelo id.
///
/// Repete o filtro de soft-delete da escrita: sem ele, um produto removido
/// reapareceria na leitura.
pub struct GetProductDql {
    /// O alvo, como `BIGINT` — o base62 já foi decodificado pelo caso de uso.
    id: i64,
}

impl GetProductDql {
    /// Monta a consulta.
    pub(crate) const fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Dql for GetProductDql {
    type View = Option<ProductViewItem>;
}

impl SqlDql for GetProductDql {
    fn build(&self) -> SqlQuery {
        Select::from("products p")
            .column(COLUMNS)
            .filter("p.id = ?", [Bind::Int(self.id)])
            .filter("p.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(ProductReader::item).transpose()
    }
}
