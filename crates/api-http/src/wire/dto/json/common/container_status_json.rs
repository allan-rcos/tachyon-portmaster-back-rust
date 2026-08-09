//! O DTO de JSON de `ContainerStatus`.

use serde::{Deserialize, Serialize};

/// Em que ponto do ciclo um contêiner está.
///
/// Sai no fio como o nome da variante, que é o que o `.fbs` publica e o que
/// `swagger/swagger.json` documenta.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ContainerStatusJson {
    /// Vazio, esperando carga.
    Empty,
    /// Recebendo carga.
    Loading,
    /// Lacrado, pronto para sair.
    Sealed,
    /// A caminho.
    InTransit,
}
