//! A natureza de uma falha comum, sem falar de transporte.

/// De que tipo é a recusa que o [`AppError`] carrega.
///
/// Existe para que quem consome o erro comum não precise casar variante por
/// variante só para decidir o que fazer com ele. O agrupamento é por **natureza
/// da falha** — a validação do domínio é validação, venha ela de um usuário, de
/// um papel ou de um manifesto — e deliberadamente **não** por intenção de
/// protocolo: quem traduz isto para um status HTTP é a camada de API, e ela é a
/// única que pode, porque é a única que sabe que existe HTTP.
///
/// A diferença importa quando aparecer uma segunda apresentação. Um CLI não tem
/// 422; tem código de saída. Um consumidor de fila não tem 403; tem
/// *dead letter*. Os dois conseguem decidir a partir daqui, e nenhum precisa
/// desfazer uma tradução que o `app` já tivesse feito.
///
/// As recusas **próprias de um serviço** não passam por aqui: elas são poucas,
/// específicas, e o controller que chamou aquele caso de uso as casa uma a uma —
/// é o que faz `GET /products/{id}` responder 404 com o id que faltou em vez de
/// um "ausência" genérico.
///
/// [`AppError`]: crate::error::AppError
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorKind {
    /// O dado que chegou não passa pelas regras de forma do domínio.
    ///
    /// Sempre em lote: quem preencheu o formulário corrige tudo de uma vez.
    Validation,

    /// Quem está pedindo é conhecido, mas não tem a permissão exigida.
    Authorization,

    /// Falhou algo que não é do domínio nem do pedido — o banco, a rede, o disco.
    Internal,
}
