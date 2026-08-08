//! A lista de permissões registradas.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;

/// Monta a tabela do catálogo de permissões.
///
/// O `id` é **posicional** — um handle de consulta, como o schema descreve, e
/// não uma chave estável. O registro é um catálogo em memória preenchido no
/// boot: não há id de banco para publicar, e inventar um sugeriria uma
/// estabilidade que ele não tem.
pub(crate) struct PermissionListResponseFactory {
    /// Os slugs registrados, na ordem do catálogo.
    slugs: Vec<String>,
}

impl PermissionListResponseFactory {
    /// Monta a factory sobre os slugs registrados.
    pub(crate) const fn of(slugs: Vec<String>) -> Self {
        Self { slugs }
    }
}

impl ResponseFactory for PermissionListResponseFactory {
    type Table = fbs::metadata::PermissionListResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::metadata::PermissionListResponse {
            data: Some(
                self.slugs
                    .iter()
                    .enumerate()
                    .map(|(index, slug)| fbs::metadata::MetadataItemResponse {
                        id: Convert::count(i64::try_from(index).unwrap_or(i64::MAX)),
                        slug: Some(slug.clone()),
                    })
                    .collect(),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_lista_de_permissoes_numera_por_posicao() {
        let table = PermissionListResponseFactory::of(vec![
            "product:read".to_owned(),
            "product:create".to_owned(),
        ])
        .table()
        .expect("a tabela precisa montar");

        let data = table.data.expect("a lista tem itens");

        assert_eq!(data[0].id, 0);
        assert_eq!(data[0].slug.as_deref(), Some("product:read"));
        assert_eq!(data[1].id, 1);
    }
}
