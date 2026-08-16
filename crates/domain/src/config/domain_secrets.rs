//! A identidade de deploy desta instância.

/// Segredos de runtime do `domain`.
///
/// Só identidade de deploy. A **estratégia** de geração de id é feature de
/// build, e a era do gerador é constante — dois deploys com epochs diferentes
/// emitiriam ids que se sobrepõem no tempo. O que varia entre instâncias é
/// apenas quem elas são.
/// O padrão é a instância zero, e ele mora aqui.
///
/// É o que vale quando o ambiente não traz as variáveis, e o elo de config o lê
/// daqui em vez de repetir o número — um segundo lugar com o mesmo padrão é um
/// lugar onde ele pode divergir. Serve a uma instalação de um processo só;
/// quem roda mais de uma **precisa** preencher os dois, senão duas instâncias
/// emitem ids que se sobrepõem.
#[derive(Debug, Clone, Copy, Default)]
pub struct DomainSecrets {
    /// Identifica o cluster na composição do Snowflake. Faixa: 0–31.
    pub cluster_id: i32,

    /// Identifica o servidor dentro do cluster. Faixa: 0–31.
    pub server_id: i32,
}
