//! A identidade de deploy desta instância.

/// Segredos de runtime do `domain`.
///
/// Só identidade de deploy. A **estratégia** de geração de id é feature de
/// build, e a era do gerador é constante — dois deploys com epochs diferentes
/// emitiriam ids que se sobrepõem no tempo. O que varia entre instâncias é
/// apenas quem elas são.
#[derive(Debug, Clone, Copy)]
pub struct DomainSecrets {
    /// Identifica o cluster na composição do Snowflake. Faixa: 0–31.
    pub cluster_id: i32,

    /// Identifica o servidor dentro do cluster. Faixa: 0–31.
    pub server_id: i32,
}
