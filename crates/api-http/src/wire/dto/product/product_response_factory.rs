//! Um produto.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ProductViewItem;

/// Monta a tabela de um produto.
pub(crate) struct ProductResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: ProductViewItem,
}

impl ProductResponseFactory {
    /// Monta a factory sobre a View, que é o que a leitura devolve.
    pub(crate) const fn of(source: ProductViewItem) -> Self {
        Self { source }
    }

    /// Monta a factory sobre o objeto de domínio, que é o que a escrita devolve.
    ///
    /// Duas entradas e uma factory só: a mensagem é a mesma, e ter duas
    /// factories seria a chance de a resposta de `POST` divergir da de `GET`.
    pub(crate) fn of_domain(product: &dyn portmaster_app::domain::Product) -> Self {
        Self {
            source: ProductViewItem {
                id: product.id().to_owned(),
                name: product.name().to_owned(),
                density: product.density(),
                risk_class: product.risk_class().as_i32(),
            },
        }
    }
}

impl ResponseFactory for ProductResponseFactory {
    type Table = fbs::product::ProductResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::product::ProductResponse {
            id: Some(self.source.id.clone()),
            name: Some(self.source.name.clone()),
            density: self.source.density,
            risk_class: Convert::risk_class(self.source.risk_class),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_view_de_produto_atravessa_inteira() {
        let table = ProductResponseFactory::of(ProductViewItem {
            id: "aZ3".into(),
            name: "Cimento".into(),
            density: 1.44,
            risk_class: 2,
        })
        .table()
        .expect("a tabela precisa montar");

        assert_eq!(table.id.as_deref(), Some("aZ3"));
        assert_eq!(table.name.as_deref(), Some("Cimento"));
        assert_eq!(table.density, 1.44);
        assert_eq!(
            table.risk_class,
            fbs::common::RiskClass::Class3FlammableLiquids
        );
    }
}
