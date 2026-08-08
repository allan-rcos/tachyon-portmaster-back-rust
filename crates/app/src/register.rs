//! O construtor do sistema.

use crate::config::AppSecrets;
use crate::interno::app_provider::AppProviderImpl;
use crate::interno::declare_metadata::declare_metadata;
use crate::provider::AppProvider;

/// Inicializa o sistema inteiro e devolve o provider da aplicação.
///
/// Encadeia os `register` das camadas de baixo e **embute** os subproviders.
/// Não há composition root: o `main` chama isto e recebe algo pronto, sem
/// conhecer `domain` nem `infra`.
///
/// ## O que acontece aqui e em nenhum outro lugar
///
/// O **catálogo de permissões** e o **grupo de marcador da sessão** são
/// registrados no boot. Os dois precisam existir antes da primeira requisição:
/// o `POST /setup` concede ao primeiro papel tudo que estiver registrado, e o
/// repositório de marcadores recusa marcar num grupo desconhecido.
///
/// Registrar aqui, e não no construtor de cada caso de uso como o PHP fazia,
/// evita repetir o registro a cada requisição — os casos de uso são
/// reconstruídos o tempo todo, o catálogo não.
pub async fn register(secrets: AppSecrets) -> anyhow::Result<impl AppProvider> {
    let domain = portmaster_domain::register(secrets.domain);
    let infra = portmaster_infra::register(secrets.infra).await?;

    declare_metadata(&domain, &infra).await?;

    Ok(AppProviderImpl::new(domain, infra))
}
