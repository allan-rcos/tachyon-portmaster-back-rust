//! A orquestração de marcadores.

use portmaster_domain::table_modules::{MarkerGroupTM, MarkerTM};
use portmaster_infra::repository::{MarkerGroupRepository, MarkerRepository};

use crate::commands::marker::{RegisterMarkerGroupCommand, SetMarkerCommand};
use crate::error::MarkerError;
use crate::queries::marker::GetMarkerQuery;
use crate::services::MarkService;

/// Monta o caso de uso de marcação.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn mark_service<T, G, R, GR>(
    marker_tm: T,
    marker_group_tm: G,
    markers: R,
    groups: GR,
) -> impl MarkService + Sync + Clone + use<T, G, R, GR> + 'static
where
    T: MarkerTM + Send + Sync + Clone + 'static,
    G: MarkerGroupTM + Send + Sync + Clone + 'static,
    R: MarkerRepository + Send + Sync + Clone + 'static,
    GR: MarkerGroupRepository + Send + Sync + Clone + 'static,
{
    MarkServiceImpl {
        marker_tm,
        marker_group_tm,
        markers,
        groups,
    }
}

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct MarkServiceImpl<T, G, R, GR> {
    /// As regras de marcador.
    marker_tm: T,
    /// As regras de grupo de marcador.
    marker_group_tm: G,
    /// Persistência dos marcadores.
    markers: R,
    /// Persistência dos grupos.
    groups: GR,
}

impl<T, G, R, GR> MarkService for MarkServiceImpl<T, G, R, GR>
where
    T: MarkerTM + Send + Sync,
    G: MarkerGroupTM + Send + Sync,
    R: MarkerRepository + Send + Sync,
    GR: MarkerGroupRepository + Send + Sync,
{
    /// Declara um grupo de marcador.
    ///
    /// Sem transação: os grupos vivem num registro em memória, e envolvê-los
    /// numa transação abriria uma conexão de banco para não consultar banco
    /// nenhum. Sem checagem de permissão: quem chama é o boot, e não há
    /// chamador.
    async fn register_group(&self, command: RegisterMarkerGroupCommand) -> Result<(), MarkerError> {
        let group = self.marker_group_tm.create(command.slug)?;

        self.groups.register(group.as_ref()).await?;

        Ok(())
    }

    /// Grava ou apaga um marcador.
    ///
    /// As regras de transição — remarcar o que já vale, revalidar o que foi
    /// invalidado — são da `infra`, que é dona do estado. Reimplementá-las aqui
    /// daria ao sistema duas opiniões sobre a mesma coisa.
    async fn set(&self, command: SetMarkerCommand) -> Result<(), MarkerError> {
        let marker = self
            .marker_tm
            .create(command.group, &command.value, command.flag)?;

        self.markers
            .put(marker.as_ref(), command.ttl_seconds)
            .await?;

        Ok(())
    }

    /// Cria um marcador só para chegar ao digest: é o `TableModule` que sabe
    /// reduzir o valor em claro à chave, e duplicar essa conversão aqui faria
    /// as duas divergirem no dia em que o hash mudasse.
    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, MarkerError> {
        let marker = self.marker_tm.create(query.group, &query.value, false)?;

        Ok(self.markers.is_valid(marker.group(), marker.key()).await?)
    }
}

#[cfg(test)]
#[path = "tests/mark_service_impl_test.rs"]
mod tests;
