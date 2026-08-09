//! A natureza de uma falha, sem falar de transporte.

/// De que tipo é a recusa que um caso de uso devolveu.
///
/// Existe para que quem consome o [`AppError`] não precise casar as onze
/// variantes uma a uma só para decidir o que fazer com o erro. O agrupamento é
/// por **natureza da falha** — a validação do domínio é validação, venha ela de
/// um usuário, de um papel ou de um manifesto — e deliberadamente **não** por
/// intenção de protocolo: quem traduz isto para um status HTTP é a camada de
/// API, e ela é a única que pode, porque é a única que sabe que existe HTTP.
///
/// A diferença importa quando aparecer uma segunda apresentação. Um CLI não tem
/// 422; tem código de saída. Um consumidor de fila não tem 404; tem
/// *dead letter*. Os dois conseguem decidir a partir daqui, e nenhum precisa
/// desfazer uma tradução que o `app` já tivesse feito.
///
/// [`AppError`]: crate::error::AppError
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorKind {
    /// O dado que chegou não passa pelas regras de forma do domínio.
    ///
    /// Sempre em lote: quem preencheu o formulário corrige tudo de uma vez.
    Validation,

    /// Quem está pedindo não provou ser quem diz — ou a prova não vale mais.
    Authentication,

    /// Quem está pedindo é conhecido, mas não tem a permissão exigida.
    Authorization,

    /// O recurso endereçado não existe.
    Absence,

    /// A operação é bem formada e permitida, mas contradiz o estado atual.
    ///
    /// Selar um contêiner que não está carregando é o exemplo: nada no pedido
    /// está errado, o pátio é que não está no ponto de aceitá-lo.
    Rule,

    /// Falhou algo que não é do domínio nem do pedido — o banco, a rede, o disco.
    Internal,
}
