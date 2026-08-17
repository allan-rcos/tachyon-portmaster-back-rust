//! A orquestração do painel.
//!
//! ## As permissões são privadas
//!
//! Os slugs abaixo são **contrato**: já existem em papéis gravados no banco de
//! quem roda a versão PHP, e renomear qualquer um revoga silenciosamente o
//! acesso de quem o tinha. São `const` privadas porque uma permissão pertence a
//! exatamente um caso de uso — é ele quem a compara com o `UserContext`, e não
//! há segundo lugar no sistema que precise vê-la. O boot as registra chamando
//! `declare_permissions`, sem nunca lê-las.

use portmaster_infra::query::views::MetricsView;
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::ViewCacheRepository;
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::metadata::RegisterPermissionCommand;
use crate::error::{AppError, MetricsError};
use crate::event::{MetaEvent, MetaEventStackPublisher};
use crate::queries::metrics::GetMetricsQuery;
use crate::services::MetadataService;
use crate::services::MetricsService;

/// Ler o painel do pátio.
const READ: &str = "metrics:read";

/// O grupo que este serviço lê. Nada o invalida: o painel vive do TTL, porque
/// toda escrita do sistema o afeta e derrubá-lo a cada uma o deixaria sempre frio.
const CACHE_GROUP: &str = "metrics";

/// Monta o caso de uso de painel.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn metrics_service<Q, C, E>(
    queries: Q,
    views: C,
    events: E,
) -> impl MetricsService + Sync + Clone + use<Q, C, E> + 'static
where
    Q: QueryRepository + Send + Sync + Clone + 'static,
    C: ViewCacheRepository + Send + Sync + Clone + 'static,
    E: MetaEventStackPublisher + Send + Sync + Clone + 'static,
{
    MetricsServiceImpl {
        queries,
        views,
        events,
    }
}

/// A chave do painel.
///
/// Uma só: a leitura não tem parâmetro. Nenhuma escrita a derruba — o painel
/// conta produtos e contêineres que outros serviços mexem, e fazer cada um deles
/// invalidar `metrics:` os obrigaria a saber quem os lê. O que limita o dado
/// velho aqui é o TTL do cache de leitura, na `infra`.
/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct MetricsServiceImpl<Q, C, E> {
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
    /// Onde este caso de uso registra o que a borda precisa saber.
    events: E,
}

impl<Q, C, E> MetricsService for MetricsServiceImpl<Q, C, E>
where
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
    E: MetaEventStackPublisher + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), MetricsError> {
        for slug in [READ] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    /// O painel do pátio, atrás do cache de leitura.
    ///
    /// É a leitura que mais se beneficia do cache — oito agregações varrendo as
    /// tabelas inteiras, pedidas a cada carregamento de tela.
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, MetricsError> {
        if !query.context.has_permission(READ) {
            return Err(AppError::permission_denied(READ).into());
        }

        let dql = dql::metrics();
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            self.events.emit(MetaEvent::ViewCacheHit);
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, MetricsError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }
}

#[cfg(test)]
#[path = "tests/metrics_service_impl_test.rs"]
mod tests;
