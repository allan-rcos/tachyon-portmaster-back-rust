//! Registrar uma permissão.

/// Declarar uma permissão que o sistema reconhece.
///
/// Roda no boot, uma vez por slug. Sem `UserContext` pelo mesmo motivo do
/// [`RegisterMarkerGroupCommand`](crate::commands::marker::RegisterMarkerGroupCommand):
/// não há chamador, e o `POST /setup` concede ao primeiro papel tudo que estiver
/// registrado quando ele rodar.
#[derive(Debug, Clone)]
pub struct RegisterPermissionCommand {
    /// O slug da permissão, no formato `domain:action`.
    pub slug: String,
}
