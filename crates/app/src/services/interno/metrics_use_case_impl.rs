//! A orquestração do painel.

use crate::cache::cache_key::CacheKey;
use crate::cache::read_through::ReadThrough;
use crate::error::AppError;
use crate::queries::metrics::GetMetricsQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::MetricsUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::MetricsView;
use portmaster_infra::query::{QueryFactory, QueryRepository};

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MetricsUseCaseImpl<Q, F, C, U> {
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// De onde os DQLs saem, já com os parâmetros.
    dqls: F,
    /// O cache de leitura, para o read-through e a invalidação.
    cache: C,
    /// Quem abre e fecha a transação.
    unit_of_work: U,
    /// A permissão exigida para read.
    read_permission: RequiresPermission,
}

impl<Q, F, C, U> MetricsUseCaseImpl<Q, F, C, U> {
    /// Monta o caso de uso, declarando a permissão que ele exige.
    pub(crate) const fn new(queries: Q, dqls: F, cache: C, unit_of_work: U) -> Self {
        Self {
            queries,
            dqls,
            cache,
            unit_of_work,
            read_permission: RequiresPermission::new(PermissionSlug::METRICS_READ),
        }
    }
}

impl<Q, F, C, U> MetricsUseCase for MetricsUseCaseImpl<Q, F, C, U>
where
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// O painel do pátio, atrás do cache de leitura.
    ///
    /// Não tem parâmetro: uma chave só, e é justamente a leitura que mais se
    /// beneficia do cache — oito agregações varrendo as tabelas inteiras,
    /// pedidas a cada carregamento de tela.
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = CacheKey::of(CacheKey::METRICS, "get", &[]);

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.metrics();

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
