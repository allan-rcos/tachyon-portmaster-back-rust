//! O vocabulário de `ContainerStatus`, independente de formato.

use crate::wire::dto::json::common::container_status_json::ContainerStatusJson;
use crate::wire::tables as fbs;

/// Em que ponto do ciclo um contêiner está.
///
/// O conjunto é fechado e publicado em `common.fbs`: cliente e servidor
/// compartilham o mesmo vocabulário. Este enum é a forma dele que não
/// depende de formato — os dois DTOs saem daqui, e nenhum deles é este.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ContainerStatusX {
    /// Vazio, esperando carga.
    #[default]
    Empty,
    /// Recebendo carga.
    Loading,
    /// Lacrado, pronto para sair.
    Sealed,
    /// A caminho.
    InTransit,
}

impl ContainerStatusX {
    /// O valor a partir do índice que a View carrega.
    ///
    /// Um índice fora da faixa cai no valor neutro em vez de derrubar a
    /// resposta: a linha guardada no banco não é entrada do cliente, e um
    /// registro estranho não deve custar a página inteira a quem a pediu.
    pub(crate) const fn of_index(index: i32) -> Self {
        match index {
            1 => Self::Loading,
            2 => Self::Sealed,
            3 => Self::InTransit,
            _ => Self::Empty,
        }
    }

    /// O valor na tabela do planus.
    pub(crate) const fn to_fbs(self) -> fbs::common::ContainerStatus {
        match self {
            Self::Empty => fbs::common::ContainerStatus::Empty,
            Self::Loading => fbs::common::ContainerStatus::Loading,
            Self::Sealed => fbs::common::ContainerStatus::Sealed,
            Self::InTransit => fbs::common::ContainerStatus::InTransit,
        }
    }

    /// O valor no DTO de JSON.
    pub(crate) const fn to_json(self) -> ContainerStatusJson {
        match self {
            Self::Empty => ContainerStatusJson::Empty,
            Self::Loading => ContainerStatusJson::Loading,
            Self::Sealed => ContainerStatusJson::Sealed,
            Self::InTransit => ContainerStatusJson::InTransit,
        }
    }
}
