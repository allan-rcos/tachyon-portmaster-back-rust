//! O contrato do logger que as camadas carregam.

/// Um logger nomeado, com campos acumulados.
///
/// É uma trait e não uma struct porque nenhuma camada acima deve saber para
/// onde o log vai. Hoje vai para o `tracing`; se amanhã for para um coletor, o
/// `app` e as apresentações não mudam uma linha. É a mesma razão pela qual só a
/// `infra` depende de `tracing` — e por que ninguém acima dela chama uma macro
/// de log direto.
///
/// Não há `dyn` aqui: a impl é única e chega pelo
/// [`LoggerFactory`](crate::logging::LoggerFactory) como `impl Logger`.
pub trait Logger: Clone + Send + Sync + 'static {
    /// Um logger igual a este, mais um campo.
    ///
    /// Devolve um logger **novo** em vez de alterar o corrente: é o que permite
    /// carimbar o `request_id` de uma requisição sem que ele vaze para as outras
    /// que compartilham o mesmo logger base.
    #[must_use]
    fn with_field(&self, key: &str, value: impl Into<String>) -> Self
    where
        Self: Sized;

    /// Registra um evento de rotina.
    fn info(&self, message: &str);

    /// Registra algo suspeito que não impediu a operação.
    fn warn(&self, message: &str);

    /// Registra uma falha.
    fn error(&self, message: &str);

    /// Registra detalhe que só interessa quando se está investigando.
    fn debug(&self, message: &str);

    /// O nome do componente que este logger identifica.
    fn name(&self) -> &str;
}
