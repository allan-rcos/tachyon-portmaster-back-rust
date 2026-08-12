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
    /// Acrescenta um campo ao contexto da tarefa corrente.
    ///
    /// Não devolve logger novo, e não altera este: o campo não pertence ao
    /// logger, pertence à **tarefa**. Fica guardado no span que o transporte
    /// abriu, então toda linha emitida dali para baixo o carrega — inclusive as
    /// que outro componente escrever com outro logger, que nunca soube do
    /// assunto. Era o que carimbar o campo no logger não conseguia fazer: só
    /// alcançava quem tivesse aquele logger em mãos.
    ///
    /// Fora de um span aberto o campo é descartado em silêncio. Não há onde
    /// guardá-lo, e recusar a linha por isso seria pior do que perdê-lo.
    fn with_field(&self, key: &str, value: impl Into<String>);

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
