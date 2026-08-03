//! As consultas de produto.

use portmaster_domain::enums::RiskClass;
use sqlx::mysql::MySqlRow;

use super::{effective_limit, like, normalized_search};
use crate::query::cursor::{filters, Cursor, CursorFilters};
use crate::query::row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{ProductListView, ProductViewItem};
use crate::query::{Dql, ListParams, SqlDql};

/// As colunas que a View de produto precisa.
///
/// Nomeadas em vez de `*`: a projeção é o contrato da hidratação, e um `SELECT *`
/// faria uma coluna nova entrar na consulta sem que ninguém a pedisse.
const COLUMNS: &str = "p.id, p.name, p.density, p.risk_class";

/// Um produto pelo id.
pub(crate) struct GetProductDql {
    id: i64,
}

impl GetProductDql {
    /// Monta a consulta.
    pub(crate) fn new(id: i64) -> Self {
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
            // O mesmo filtro de soft-delete da escrita: sem ele, um produto
            // removido reapareceria na leitura.
            .filter("p.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(item).transpose()
    }
}

/// A listagem de produtos.
pub(crate) struct ListProductsDql {
    params: ListParams,
}

impl ListProductsDql {
    /// Monta a consulta.
    pub(crate) fn new(params: ListParams) -> Self {
        Self { params }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        filters([
            ("limit", self.limit().to_string()),
            (
                "search",
                normalized_search(self.params.search.as_deref()).unwrap_or_default(),
            ),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        effective_limit(self.params.limit)
    }
}

impl Dql for ListProductsDql {
    type View = ProductListView;
}

impl SqlDql for ListProductsDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let search = normalized_search(self.params.search.as_deref());
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        // A contagem repete o filtro de busca porque o total tem que descrever o
        // conjunto de onde a página sai. Contar sem o filtro reportaria o
        // catálogo inteiro numa busca por uma palavra só.
        let (total_sql, total_binds) = match &search {
            Some(term) => (
                "(SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total",
                vec![Bind::Text(like(term))],
            ),
            None => (
                "(SELECT COUNT(*) FROM products WHERE deleted_at IS NULL) AS _total",
                Vec::new(),
            ),
        };

        let mut select = Select::from("products p")
            .column(COLUMNS)
            .column_bound(total_sql, total_binds)
            // Keyset: a página seguinte começa depois do último id servido, e
            // inserções no meio-tempo não deslocam nada.
            .filter("p.id > ?", [Bind::Int(last_id)])
            .filter("p.deleted_at IS NULL", [])
            .order_by("p.id ASC")
            .limit(limit);

        if let Some(term) = &search {
            select = select.filter("p.search_name LIKE ?", [Bind::Text(like(term))]);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(item(row)?);
            last_id = row::number(row, "id")?;
            total = row::number(row, "_total")?;
        }

        Ok(ProductListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

/// Uma linha de `products` como a View a quer.
fn item(row: &MySqlRow) -> anyhow::Result<ProductViewItem> {
    Ok(ProductViewItem {
        id: row::id(row, "id")?,
        name: row::text(row, "name")?,
        density: row::real(row, "density")?,
        risk_class: row::enum_index(row, "risk_class", RiskClass::from_i32, "RiskClass")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_busca_entra_na_pagina_e_na_contagem() {
        // O total tem que descrever o mesmo conjunto que a página percorre —
        // senão uma busca por uma palavra reportaria o catálogo inteiro.
        let query = ListProductsDql::new(ListParams {
            search: Some("Cimento".into()),
            ..ListParams::default()
        })
        .build();

        assert_eq!(
            query.sql,
            "SELECT p.id, p.name, p.density, p.risk_class, \
             (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total \
             FROM products p \
             WHERE p.id > ? AND p.deleted_at IS NULL AND p.search_name LIKE ? \
             ORDER BY p.id ASC LIMIT 20"
        );
        assert_eq!(
            query.binds,
            vec![
                Bind::Text("%cimento%".into()),
                Bind::Int(0),
                Bind::Text("%cimento%".into()),
            ],
            "o bind da contagem vem primeiro, porque o `?` dela sai antes no texto"
        );
    }

    #[test]
    fn sem_busca_nao_ha_filtro_de_texto() {
        let query = ListProductsDql::new(ListParams::default()).build();

        assert!(!query.sql.contains("LIKE"));
        assert_eq!(query.binds, vec![Bind::Int(0)]);
    }

    #[test]
    fn o_cursor_move_o_piso_da_varredura() {
        let params = ListParams {
            limit: Some(5),
            ..ListParams::default()
        };
        let dql = ListProductsDql::new(params.clone());
        let token =
            Cursor::next(5, 5, 4_242, &dql.cursor_filters()).expect("página cheia emite cursor");

        let seguinte = ListProductsDql::new(ListParams {
            cursor: Some(token),
            ..params
        })
        .build();

        assert_eq!(seguinte.binds, vec![Bind::Int(4_242)]);
    }

    #[test]
    fn um_cursor_de_outra_busca_recomeca_do_zero() {
        // Trocar o termo e reenviar o cursor antigo continuaria a varredura do
        // conjunto anterior sob o filtro novo.
        let dql = ListProductsDql::new(ListParams {
            search: Some("cimento".into()),
            ..ListParams::default()
        });
        let token =
            Cursor::next(20, 20, 900, &dql.cursor_filters()).expect("página cheia emite cursor");

        let outra = ListProductsDql::new(ListParams {
            cursor: Some(token),
            search: Some("areia".into()),
            ..ListParams::default()
        })
        .build();

        assert!(
            outra.binds.contains(&Bind::Int(0)),
            "o cursor incompatível deveria ter sido ignorado: {:?}",
            outra.binds
        );
    }

    #[test]
    fn o_limite_ausente_ou_zero_cai_no_padrao() {
        for limit in [None, Some(0)] {
            let query = ListProductsDql::new(ListParams {
                limit,
                ..ListParams::default()
            })
            .build();

            assert!(
                query.sql.ends_with("LIMIT 20"),
                "limite {limit:?} virou {}",
                query.sql
            );
        }
    }

    #[test]
    fn a_busca_por_id_filtra_o_soft_delete() {
        let query = GetProductDql::new(99).build();

        assert_eq!(
            query.sql,
            "SELECT p.id, p.name, p.density, p.risk_class FROM products p \
             WHERE p.id = ? AND p.deleted_at IS NULL LIMIT 1"
        );
        assert_eq!(query.binds, vec![Bind::Int(99)]);
    }
}
