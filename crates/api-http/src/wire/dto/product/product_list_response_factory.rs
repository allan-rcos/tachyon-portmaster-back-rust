//! A listagem de produtos.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::dto::product::product_response_factory::ProductResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ProductListView;

/// Monta a tabela da listagem.
pub(crate) struct ProductListResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: ProductListView,
}

impl ProductListResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: ProductListView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for ProductListResponseFactory {
    type Table = fbs::product::ProductListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::product::ProductListResponse {
            next_cursor: self.source.next_cursor.clone(),
            data: Some(
                self.source
                    .items
                    .iter()
                    .map(|item| ProductResponseFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            total: Convert::count(self.source.total),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portmaster_app::views::ProductViewItem;
    use pretty_assertions::assert_eq;

    /// O cursor e o total são o que faz a próxima página ser pedível;
    /// perdê-los no mapeamento deixaria o cliente preso na primeira.
    #[test]
    fn a_listagem_leva_cursor_e_total() {
        let table = ProductListResponseFactory::of(ProductListView {
            items: vec![ProductViewItem {
                id: "aZ3".into(),
                name: "Cimento".into(),
                density: 1.0,
                risk_class: 0,
            }],
            next_cursor: Some("abc".into()),
            total: 42,
        })
        .table()
        .expect("a tabela precisa montar");

        assert_eq!(table.next_cursor.as_deref(), Some("abc"));
        assert_eq!(table.total, 42);
        assert_eq!(table.data.map(|data| data.len()), Some(1));
    }
}
