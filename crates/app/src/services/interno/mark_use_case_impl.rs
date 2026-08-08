//! A orquestração de marcadores.

use crate::commands::marker::SetMarkerCommand;
use crate::error::AppError;
use crate::queries::marker::GetMarkerQuery;
use crate::services::MarkUseCase;
use portmaster_domain::table_modules::MarkerTM;
use portmaster_infra::repository::MarkerRepository;

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MarkUseCaseImpl<T, R> {
    /// As regras de marcador.
    marker_tm: T,
    /// Persistência dos marcadores.
    markers: R,
}

impl<T, R> MarkUseCaseImpl<T, R> {
    /// Monta o caso de uso.
    pub(crate) const fn new(marker_tm: T, markers: R) -> Self {
        Self { marker_tm, markers }
    }
}

impl<T: MarkerTM + Send + Sync, R: MarkerRepository + Send + Sync> MarkUseCase
    for MarkUseCaseImpl<T, R>
{
    /// Grava ou apaga um marcador.
    ///
    /// As regras de transição — remarcar o que já vale, revalidar o que foi
    /// invalidado — são da `infra`, que é dona do estado. Reimplementá-las aqui
    /// daria ao sistema duas opiniões sobre a mesma coisa.
    async fn set(&self, command: SetMarkerCommand) -> Result<(), AppError> {
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
    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, AppError> {
        let marker = self.marker_tm.create(query.group, &query.value, false)?;

        Ok(self.markers.is_valid(marker.group(), marker.key()).await?)
    }
}
