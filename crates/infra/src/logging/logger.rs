//! O contrato do logger que as camadas carregam.

/// Um logger nomeado, com campos acumulados.
///
/// É uma trait e não uma struct porque nenhuma camada acima deve saber para
/// onde o log vai. Hoje vai para o `tracing`; se amanhã for para um coletor, o
/// `app` e as apresentações não mudam uma linha. É a mesma razão pela qual só a
/// `infra` emite evento de log — e por que ninguém acima dela chama uma macro
/// de log direto.
///
/// Não há `dyn` aqui: a impl é única e chega pelo
/// [`LoggerFactory`](crate::logging::LoggerFactory) como `impl Logger`.
///
/// ## Os campos do instante são um array, não um logger novo
///
/// Cada método recebe `[(&str, &str); N]` com o que só vale para **aquela**
/// linha. O `N` é constante, então o array vive na pilha e some na
/// monomorfização; antes cada campo de ocasião custava um logger clonado e uma
/// entrada num mapa, jogados fora na linha seguinte. Sem campo nenhum, é `[]`.
pub trait Logger: Clone + Send + Sync + 'static {
    /// Um logger igual a este, mais um campo.
    ///
    /// Devolve um logger **novo** em vez de alterar o corrente. É para o que é
    /// identidade **do logger** e se repete em toda linha que ele escrever; o
    /// que pertence à requisição não passa por aqui, e sim pelo span que o
    /// transporte abre.
    #[must_use]
    fn with_field(&self, key: &str, value: impl Into<String>) -> Self
    where
        Self: Sized;

    /// Registra um evento de rotina.
    fn info<const N: usize>(&self, message: &str, fields: [(&str, &str); N]);

    /// Registra algo suspeito que não impediu a operação.
    fn warn<const N: usize>(&self, message: &str, fields: [(&str, &str); N]);

    /// Registra uma falha.
    fn error<const N: usize>(&self, message: &str, fields: [(&str, &str); N]);

    /// Registra detalhe que só interessa quando se está investigando.
    fn debug<const N: usize>(&self, message: &str, fields: [(&str, &str); N]);

    /// O nome do componente que este logger identifica.
    fn name(&self) -> &str;
}
