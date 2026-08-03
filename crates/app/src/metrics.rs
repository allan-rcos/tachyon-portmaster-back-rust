//! O painel do pátio.

use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::MetricsView;
use portmaster_infra::query::{QueryFactory, QueryRepository};

use crate::authorization::{slug, RequiresPermission};
use crate::cache::{self, prefix};
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Ler o painel.
#[derive(Debug, Clone)]
pub struct GetMetricsQuery {
    /// Quem está consultando.
    pub context: UserContext,
}

/// O que a apresentação pode pedir sobre o painel.
#[trait_variant::make(Send)]
pub trait MetricsUseCase {
    /// As oito agregações do pátio.
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MetricsUseCaseImpl<Q, F, C, U> {
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
    read_permission: RequiresPermission,
}

impl<Q, F, C, U> MetricsUseCaseImpl<Q, F, C, U> {
    /// Monta o caso de uso, declarando a permissão que ele exige.
    pub(crate) fn new(queries: Q, dqls: F, cache: C, unit_of_work: U) -> Self {
        Self {
            queries,
            dqls,
            cache,
            unit_of_work,
            read_permission: RequiresPermission::new(slug::METRICS_READ),
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
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, AppError> {
        self.read_permission.authorize(&query.context)?;

        // O painel não tem parâmetro: uma chave só, e é justamente a leitura que
        // mais se beneficia do cache — oito agregações varrendo as tabelas
        // inteiras, pedidas a cada carregamento de tela.
        let key = cache::key(prefix::METRICS, "get", &[]);

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.metrics();

            transaction(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
