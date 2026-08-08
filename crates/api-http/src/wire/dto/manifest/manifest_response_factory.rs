//! O que um embarque ou desembarque responde.

use crate::error::api_error::ApiError;
use crate::wire::dto::container::container_response_factory::ContainerResponseFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;

/// Monta a tabela do resultado de um movimento de carga.
///
/// Responde com o **contêiner depois do movimento**, e não com a linha do
/// manifesto: quem embarcou quer saber o peso e o status resultantes, e pedi-los
/// numa segunda requisição seria uma corrida contra outro embarque.
///
/// O contêiner entra como **factory**, não como View: o movimento devolve um
/// objeto de domínio, e é a `ContainerResponseFactory` que sabe montar a tabela
/// a partir de qualquer uma das duas origens.
pub(crate) struct ManifestResponseFactory {
    message: &'static str,
    container: ContainerResponseFactory,
}

impl ManifestResponseFactory {
    /// Monta a factory com a mensagem e o contêiner resultante.
    pub(crate) const fn of(message: &'static str, container: ContainerResponseFactory) -> Self {
        Self { message, container }
    }
}

impl ResponseFactory for ManifestResponseFactory {
    type Table = fbs::manifest::ManifestResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::manifest::ManifestResponse {
            message: Some(self.message.to_owned()),
            container: Some(Box::new(self.container.table()?)),
        })
    }
}
