//! A listagem paginada de produtos.

use anyhow::{anyhow, Context as _};
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{QueryBuilder, Row as _};

use crate::entity::codec::Codec;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::paging::Paging;
use crate::query::views::{ProductListView, ProductViewItem};
use crate::query::{Dql, SqlDql};
use portmaster_domain::enums::RiskClass;

/// As colunas que a View de produto precisa.
///
/// Nomeadas em vez de `*`: a projeção é o contrato da hidratação, e um `SELECT *`
/// faria uma coluna nova entrar na consulta sem que ninguém a pedisse.
const COLUMNS: &str = "p.id, p.name, p.density, p.risk_class";

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
pub(super) fn read_item(row: &MySqlRow) -> anyhow::Result<ProductViewItem> {
    let risk_class: i64 = row
        .try_get("risk_class")
        .context("coluna `risk_class` não veio como inteiro")?;
    let risk_class = i32::try_from(risk_class)
        .with_context(|| format!("coluna `risk_class` guarda {risk_class}, fora da faixa"))?;

    RiskClass::from_i32(risk_class)
        .ok_or_else(|| anyhow!("{risk_class} não corresponde a variante nenhuma de RiskClass"))?;

    Ok(ProductViewItem {
        id: Codec::encode_id(
            row.try_get("id")
                .context("coluna `id` não veio como inteiro")?,
        ),
        name: row
            .try_get("name")
            .context("coluna `name` não veio como texto")?,
        density: row
            .try_get("density")
            .context("coluna `density` não veio como real")?,
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
    fn build(&self) -> QueryBuilder<MySql> {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(COLUMNS);

        builder.push(", (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL");
        if let Some(term) = &self.search {
            builder.push(" AND search_name LIKE ");
            builder.push_bind(Paging::like(term));
        }
        builder.push(") AS _total");

        builder.push(" FROM products p WHERE p.id > ");
        builder.push_bind(last_id);
        builder.push(" AND p.deleted_at IS NULL");

        if let Some(term) = &self.search {
            builder.push(" AND p.search_name LIKE ");
            builder.push_bind(Paging::like(term));
        }

        builder.push(" ORDER BY p.id ASC LIMIT ");
        builder.push_bind(i64::from(self.limit));

        builder
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(read_item(row)?);
            last_id = row
                .try_get("id")
                .context("coluna `id` não veio como inteiro")?;
            total = row
                .try_get("_total")
                .context("coluna `_total` não veio como inteiro")?;
        }

        Ok(ProductListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// O SQL que um DQL monta, para as asserções.
    fn sql_of(dql: &impl SqlDql) -> String {
        dql.build().sql().as_str().to_owned()
    }

    /// O total tem que descrever o mesmo conjunto que a página percorre — senão
    /// uma busca por uma palavra reportaria o catálogo inteiro.
    #[test]
    fn a_busca_entra_na_pagina_e_na_contagem() {
        let dql = ListProducts {
            limit: 20,
            search: Some("cimento".into()),
            cursor: None,
        };

        assert_eq!(
            sql_of(&dql),
            "SELECT p.id, p.name, p.density, p.risk_class, \
             (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total \
             FROM products p WHERE p.id > ? AND p.deleted_at IS NULL AND p.search_name LIKE ? \
             ORDER BY p.id ASC LIMIT ?"
        );
    }

    #[test]
    fn sem_busca_nao_ha_filtro_de_texto() {
        let dql = ListProducts {
            limit: 20,
            search: None,
            cursor: None,
        };

        assert!(!sql_of(&dql).contains("LIKE"));
    }

    /// Trocar o termo e reenviar o cursor antigo continuaria a varredura do
    /// conjunto anterior sob o filtro novo.
    #[test]
    fn um_cursor_de_outra_busca_recomeca_do_zero() {
        let anterior = ListProducts {
            limit: 20,
            search: Some("cimento".into()),
            cursor: None,
        };
        let token = Cursor::next(20, 20, 900, &anterior.cursor_filters())
            .expect("página cheia emite cursor");

        let outra = ListProducts {
            limit: 20,
            search: Some("areia".into()),
            cursor: Some(token.clone()),
        };
        let mesma = ListProducts {
            limit: 20,
            search: Some("cimento".into()),
            cursor: Some(token),
        };

        assert_eq!(
            Cursor::last_id_or_start(outra.cursor.as_deref(), &outra.cursor_filters()),
            0,
            "o cursor de outra busca deveria ter sido ignorado"
        );
        assert_eq!(
            Cursor::last_id_or_start(mesma.cursor.as_deref(), &mesma.cursor_filters()),
            900,
            "o cursor da mesma busca move o piso da varredura"
        );
    }

    #[test]
    fn o_limite_ausente_ou_zero_cai_no_padrao() {
        for limit in [None, Some(0)] {
            assert_eq!(
                Paging::effective_limit(limit),
                crate::query::DEFAULT_LIMIT,
                "limite {limit:?} deveria cair no padrão"
            );
        }
    }
}
