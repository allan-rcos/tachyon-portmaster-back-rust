//! O contrato de quem quer saber o id desta requisição.

/// O identificador de correlação da requisição corrente.
///
/// Leitura e nada mais. Quem **atribui** o id é o layer que abre o escopo, e ele
/// alcança o escritor por dentro do módulo — de fora não há como carimbar um id
/// diferente do que a requisição recebeu, que é a única forma de o valor no log
/// significar alguma coisa.
pub(crate) trait RequestIdPort: Clone + Send + Sync + 'static {
    /// O id desta requisição, se houver escopo.
    ///
    /// `None` quando não há — código chamado fora de uma requisição, um teste
    /// que exercita um controller direto. É informação de correlação, não
    /// autorização: faltar não é motivo para recusar nada, ao contrário do que
    /// vale para a sessão.
    fn current(&self) -> Option<String>;
}
