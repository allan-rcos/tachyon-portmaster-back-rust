//! O que `POST /auth/login` recebe.

/// As credenciais de um login.
///
/// Os campos são `Option` embora o `.fbs` os marque `required`, e isso é o ponto
/// do desenho novo: um campo ausente chega aqui como `None`, o caso de uso o
/// repassa ao `TableModule`, e o cliente recebe **422 nomeando o campo** — todos
/// os que faltaram, de uma vez. No desenho antigo o serde falhava antes, e a
/// resposta era um 400 sem dizer o quê.
#[derive(Debug, Clone, Default)]
pub(crate) struct LoginRequest {
    /// O e-mail informado.
    pub(crate) email: Option<String>,
    /// A senha em claro. Morre no `TableModule`, que guarda só o hash.
    pub(crate) password: Option<String>,
}
