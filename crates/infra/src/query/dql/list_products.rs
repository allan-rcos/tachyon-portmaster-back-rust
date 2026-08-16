//! A listagem paginada de produtos.

use anyhow::{anyhow, Context as _};
use mysql_async::{Params, Row, Value};

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{ProductListView, ProductViewItem};
use crate::query::{Dql, SqlDql};
use portmaster_domain::enums::RiskClass;

/// A listagem paginada de produtos.
///
/// O cursor chega como o cliente o mandou, ainda codificado: decodificá-lo é
/// assunto desta camada, e devolver um `Cursor` para o `app` faria a paginação
/// vazar para quem só queria pedir uma página.
pub fn list_products(
    cursor: Option<String>,
    limit: Option<u32>,
    search: Option<&str>,
) -> impl SqlDql<View = ProductListView> {
    ListProducts {
        limit: Paging::effective_limit(limit),
        search: Paging::normalized_search(search),
        cursor,
    }
}

/// Uma linha de `products` como a View a quer.
///
/// `pub(super)` porque o DQL de item lê a mesma projeção: duplicar a leitura
/// faria as duas divergirem na primeira coluna acrescentada.
pub(super) fn read_item(row: &Row) -> anyhow::Result<ProductViewItem> {
    let risk_class: i64 = Column::of(row, "risk_class")?;
    let risk_class = i32::try_from(risk_class)
        .with_context(|| format!("coluna `risk_class` guarda {risk_class}, fora da faixa"))?;

    RiskClass::from_i32(risk_class)
        .ok_or_else(|| anyhow!("{risk_class} não corresponde a variante nenhuma de RiskClass"))?;

    Ok(ProductViewItem {
        id: Codec::encode_id(Column::of(row, "id")?),
        name: Column::of(row, "name")?,
        density: Column::of(row, "density")?,
        risk_class,
    })
}

/// A listagem de produtos.
///
/// ## A contagem repete os filtros da página
///
/// O total tem que descrever o conjunto de onde a página sai. Contar sem o
/// filtro de busca reportaria o catálogo inteiro numa busca por uma palavra só.
///
/// ## A paginação é keyset, não offset
///
/// A página seguinte começa **depois do último id servido**, e inserções no
/// meio-tempo não deslocam nada — que é o defeito que o `OFFSET` tem.
struct ListProducts {
    /// O tamanho da página, já resolvido.
    limit: u32,
    /// O termo já reduzido à chave que as colunas `search_*` guardam.
    search: Option<String>,
    /// O cursor como o cliente o mandou.
    cursor: Option<String>,
}

impl ListProducts {
    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit.to_string()),
            ("search", self.search.clone().unwrap_or_default()),
        ])
    }

    /// A condição de busca, escrita uma vez e aplicada nas duas metades.
    ///
    /// A página e a contagem precisam descrever o mesmo conjunto: contar sem o
    /// filtro reportaria o catálogo inteiro numa busca por uma palavra só. O
    /// nome do parâmetro é o mesmo nas duas, e é ligado uma vez.
    fn search_condition(search: Option<&String>, alias: &str) -> String {
        search.map_or_else(String::new, |_| {
            format!(" AND {alias}search_name LIKE :search")
        })
    }

    /// Os valores que o texto da consulta nomeia.
    fn params(&self, last_id: i64) -> Params {
        let mut values = vec![
            ("last_id".to_owned(), Value::Int(last_id)),
            ("limit".to_owned(), Value::Int(i64::from(self.limit))),
        ];

        if let Some(term) = &self.search {
            values.push(("search".to_owned(), Value::from(Paging::like(term))));
        }

        values.into()
    }
}

impl Dql for ListProducts {
    type View = ProductListView;

    fn cache_key(&self) -> String {
        format!(
            "list_products:{}:{}:{}",
            self.limit,
            self.search.as_deref().unwrap_or_default(),
            self.cursor.as_deref().unwrap_or_default()
        )
    }
}

impl SqlDql for ListProducts {
    /// As colunas são nomeadas em vez de `*`: a projeção é o contrato da
    /// hidratação, e um `SELECT *` faria uma coluna nova entrar na consulta sem
    /// que ninguém a pedisse.
    fn build(&self) -> (String, Params) {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let sql = format!(
            "SELECT p.id, p.name, p.density, p.risk_class, \
             (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL{total_search}) AS _total \
             FROM products p \
             WHERE p.id > :last_id AND p.deleted_at IS NULL{page_search} \
             ORDER BY p.id ASC LIMIT :limit",
            total_search = Self::search_condition(self.search.as_ref(), ""),
            page_search = Self::search_condition(self.search.as_ref(), "p."),
        );

        (sql, self.params(last_id))
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row)?);
            last_id = Column::of(row, "id")?;
            total = Column::of(row, "_total")?;
        }

        Ok(ProductListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}
