//! Os testes de `register`.

use crate::services::intern::{
    container_service_impl, manifest_service_impl, metadata_service_impl, metrics_service_impl,
    product_service_impl, role_service_impl, user_service_impl,
};
use pretty_assertions::assert_eq;

/// Todos os slugs que o boot registra, na ordem dos serviços.
///
/// A lista só existe sob `cfg(test)`: em produção cada slug é privado do seu
/// caso de uso, e nem este arquivo o enxerga.
fn all() -> Vec<&'static str> {
    [
        container_service_impl::PERMISSIONS,
        manifest_service_impl::PERMISSIONS,
        metadata_service_impl::PERMISSIONS,
        metrics_service_impl::PERMISSIONS,
        product_service_impl::PERMISSIONS,
        role_service_impl::PERMISSIONS,
        user_service_impl::PERMISSIONS,
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
