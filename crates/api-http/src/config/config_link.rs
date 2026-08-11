//! O que um grupo de configuração declara para participar do boot.

use crate::config::boot_draft::BootDraft;
use crate::config::env_source::EnvSource;

/// A declaração de um elo na chain de configuração.
///
/// Um campo só, no mesmo molde do `ScopeLayer` da `infra`, e é o que mantém a
/// declaração de um grupo novo em uma linha.
///
/// A função é infalível de propósito. Um elo que não consegue ler alguma coisa
/// não interrompe a chain: registra a queixa no
/// [`EnvSource`](crate::config::env_source::EnvSource) e segue com o padrão,
/// para que um boot recusado nomeie tudo que está errado de uma vez. É a mesma
/// decisão do `DotEnvChain` do PHP — *"collecting all the invalid variables in
/// one pass is the point"* —, só que sem a exceção que o elo do JWT abria.
pub(crate) struct ConfigLink {
    /// Lê as variáveis deste grupo e preenche o slot dele no rascunho.
    pub(crate) read: fn(&mut EnvSource, &mut BootDraft),
}
