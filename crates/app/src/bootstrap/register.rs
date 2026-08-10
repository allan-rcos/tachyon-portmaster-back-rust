//! O construtor do sistema.

use anyhow::Context as _;

use crate::bootstrap::app_provider::AppProviderImpl;
use crate::bootstrap::provider::AppProvider;
use crate::config::AppSecrets;
use crate::services::{
    ContainerUseCase as _, ManifestUseCase as _, MetadataUseCase as _, MetricsUseCase as _,
    ProductUseCase as _, RoleUseCase as _, UserUseCase as _,
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
/// evita repetir o registro a cada requisição: os casos de uso são reconstruídos
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
    let metadata = provider.metadata_use_case();

    metadata
        .declare_permissions()
        .await
        .context("falha ao registrar as permissões de metadados")?;

    provider
        .container_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de contêiner")?;

    provider
        .manifest_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de manifesto")?;

    provider
        .metrics_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões do painel")?;

    provider
        .product_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de produto")?;

    provider
        .role_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de papel")?;

    provider
        .user_use_case()
        .declare_permissions(&metadata)
        .await
        .context("falha ao registrar as permissões de usuário")?;

    Ok(provider)
}

#[cfg(test)]
mod tests {
    use crate::services::intern::{
        container_use_case_impl, manifest_use_case_impl, metadata_use_case_impl,
        metrics_use_case_impl, product_use_case_impl, role_use_case_impl, user_use_case_impl,
    };
    use pretty_assertions::assert_eq;

    /// Todos os slugs que o boot registra, na ordem dos serviços.
    ///
    /// A lista só existe sob `cfg(test)`: em produção cada slug é privado do seu
    /// caso de uso, e nem este arquivo o enxerga.
    fn all() -> Vec<&'static str> {
        [
            container_use_case_impl::PERMISSIONS,
            manifest_use_case_impl::PERMISSIONS,
            metadata_use_case_impl::PERMISSIONS,
            metrics_use_case_impl::PERMISSIONS,
            product_use_case_impl::PERMISSIONS,
            role_use_case_impl::PERMISSIONS,
            user_use_case_impl::PERMISSIONS,
        ]
        .concat()
    }

    #[test]
    fn o_catalogo_nao_tem_slug_repetido() {
        let mut unicos = all();
        unicos.sort_unstable();
        unicos.dedup();

        assert_eq!(
            unicos.len(),
            all().len(),
            "um slug duplicado esconde uma permissão que ninguém registrou"
        );
    }

    /// O número é contrato: são as permissões que já existem em papéis
    /// gravados.
    ///
    /// Este teste quebra tanto se alguém acrescentar um caso de uso sem
    /// declarar a permissão quanto se remover uma que ainda está em uso.
    #[test]
    fn o_catalogo_tem_as_25_permissoes_do_php() {
        assert_eq!(all().len(), 25);
    }

    /// O `TableModule` de permissão recusa slug fora deste formato — melhor
    /// descobrir aqui do que ver o boot falhar.
    #[test]
    fn todo_slug_segue_o_formato_recurso_acao() {
        for slug in all() {
            let (resource, action) = slug
                .split_once(':')
                .unwrap_or_else(|| panic!("slug sem `:`: {slug}"));

            assert!(!resource.is_empty(), "recurso vazio em {slug}");
            assert!(!action.is_empty(), "ação vazia em {slug}");
        }
    }
}
