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
use crate::queries::metrics::GetMetricsQuery;
use crate::services::MetadataService;
use crate::services::MetricsService;

/// Ler o painel do pátio.
const READ: &str = "metrics:read";

/// O grupo que este serviço lê. Nada o invalida: o painel vive do TTL, porque
/// toda escrita do sistema o afeta e derrubá-lo a cada uma o deixaria sempre frio.
const CACHE_GROUP: &str = "metrics";

/// A chave do painel.
///
/// Uma só: a leitura não tem parâmetro. Nenhuma escrita a derruba — o painel
/// conta produtos e contêineres que outros serviços mexem, e fazer cada um deles
/// invalidar `metrics:` os obrigaria a saber quem os lê. O que limita o dado
/// velho aqui é o TTL do cache de leitura, na `infra`.
/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct MetricsServiceImpl<Q, C> {
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
}

impl<Q, C> MetricsServiceImpl<Q, C> {
    /// Monta o caso de uso.
    pub(crate) const fn new(queries: Q, views: C) -> Self {
        Self { queries, views }
    }
}

impl<Q, C> MetricsService for MetricsServiceImpl<Q, C>
where
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
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

/// Os slugs deste serviço, para o teste do catálogo.
///
/// `cfg(test)`: em produção nada além deste arquivo vê um slug, e é isso que se
/// quer. O teste do catálogo precisa somá-los para afirmar as 25 permissões que
/// já existem em papéis gravados, e essa é a única razão de a lista existir.
#[cfg(test)]
pub(crate) const PERMISSIONS: &[&str] = &[READ];

#[cfg(test)]
#[path = "tests/metrics_service_impl_test.rs"]
mod tests;
