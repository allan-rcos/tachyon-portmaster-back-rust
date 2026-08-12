//! O construtor do sistema.

use anyhow::Context as _;

use crate::bootstrap::app_provider::AppProviderImpl;
use crate::bootstrap::provider::AppProvider;
use crate::config::AppSecrets;
use crate::services::{
    ContainerService as _, ManifestService as _, MetadataService as _, MetricsService as _,
    ProductService as _, RoleService as _, UserService as _,
};

/// Inicializa o sistema inteiro e devolve o provider da aplicação.
///
/// Encadeia os `register` das camadas de baixo e **embute** os subproviders.
/// Não há composition root: o `main` chama isto e recebe algo pronto, sem
/// conhecer `domain` nem `infra`.
///
/// ## O catálogo de permissões nasce aqui, e este arquivo não conhece um slug
///
/// Cada serviço declara as suas, e o que este boot faz é pedir que declarem.
/// É o molde do `declarePermission` do PHP: a permissão pertence a exatamente
/// um caso de uso, e é ele quem a nomeia. Acrescentar um serviço é acrescentar
/// uma linha aqui — não editar uma lista central que ninguém lembra de manter.
///
/// O catálogo precisa existir antes da primeira requisição: o `POST /setup`
/// concede ao primeiro papel tudo que estiver registrado. Falhar aqui derruba o
/// boot de propósito — um sistema que subiu com o catálogo pela metade daria 403
/// em endpoints que deveriam funcionar, e a causa seria invisível.
///
/// Registrar no boot, e não no construtor de cada caso de uso como o PHP fazia,
/// evita repetir o registro a cada requisição: os services são reconstruídos
/// o tempo todo, o catálogo não.
///
/// **Grupo de marcador não entra aqui.** Quem usa um grupo é a apresentação, e é
/// ela quem o declara — esta camada não sabe o que é sessão.
///
/// É o único ponto em que um erro tipado vira `anyhow`: aqui é wiring de root, e
/// o que sai daqui vai para o `main`.
pub async fn register(secrets: AppSecrets) -> anyhow::Result<impl AppProvider> {
    let domain = portmaster_domain::register(secrets.domain);
    let infra = portmaster_infra::register(secrets.infra).await?;

    let provider = AppProviderImpl::new(domain, infra);
    let metadata = provider.metadata_service();

    metadata
        .declare_permissions()
        .await
        .context("falha ao registrar as permissões de metadados")?;

    provider
        .container_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de contêiner")?;

    provider
        .manifest_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de manifesto")?;

    provider
        .metrics_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões do painel")?;

    provider
        .product_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de produto")?;

    provider
        .role_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de papel")?;

    provider
        .user_service()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de usuário")?;

    Ok(provider)
}

#[cfg(test)]
#[path = "tests/register_test.rs"]
mod tests;
