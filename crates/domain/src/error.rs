//! Erros de domínio.
//!
//! Regra de negócio devolve erro **tipado**, nunca `anyhow` e nunca um status
//! HTTP: o domínio não sabe que existe HTTP. Traduzir `SealRefused` em 409 é
//! trabalho do `api-http`, e concentrar isso lá é o que permite outra
//! apresentação — uma CLI, um daemon — reagir ao mesmo erro do jeito dela.
//!
//! Validação **acumula**. Um validator não para no primeiro campo inválido: ele
//! examina todos e devolve a lista inteira, para que o cliente conserte tudo de
//! uma vez em vez de descobrir um problema por requisição.

use std::fmt;

/// Um campo que quebrou uma regra, e o porquê.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// Nome do campo, como o cliente o enviou.
    pub field: String,
    /// O que há de errado com ele.
    pub message: String,
}

impl FieldError {
    /// Monta um erro de campo.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Acumulador de erros de validação.
///
/// Existe para que o padrão "valide tudo, reclame uma vez" fique difícil de
/// violar por engano: um validator escreve `errors.add(...)` quantas vezes
/// precisar e devolve `errors.into_result(valor)` no fim.
#[derive(Debug, Default, Clone)]
pub(crate) struct Validation {
    errors: Vec<FieldError>,
}

impl Validation {
    /// Começa uma validação sem erros.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registra um campo inválido e segue examinando os demais.
    pub(crate) fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError::new(field, message));
    }

    /// Registra um campo inválido só se a condição for verdadeira.
    pub(crate) fn add_if(
        &mut self,
        condition: bool,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        if condition {
            self.add(field, message);
        }
    }

    /// Se nada quebrou, devolve o valor; senão, a lista completa de problemas.
    pub(crate) fn into_result<T>(self, value: T) -> Result<T, Vec<FieldError>> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors)
        }
    }
}

/// Falhas ao construir ou alterar um usuário.
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de usuário inválidos")]
    Validation(Vec<FieldError>),
}

/// Falhas ao construir ou alterar um papel.
#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de papel inválidos")]
    Validation(Vec<FieldError>),
}

/// Falhas ao construir ou alterar um produto.
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de produto inválidos")]
    Validation(Vec<FieldError>),
}

/// Falhas de autenticação.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Senha não confere. A mensagem não distingue e-mail inexistente de senha
    /// errada — dizer qual dos dois falhou entrega ao atacante metade da
    /// resposta.
    #[error("Invalid e-mail or password.")]
    InvalidCredentials,
}

/// Falhas ao construir um contêiner ou ao mover seu status.
///
/// Validação e conflito são coisas diferentes: a primeira diz que os dados
/// enviados estão errados, a segunda que o pátio não está num estado em que
/// aquilo faça sentido.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de contêiner inválidos")]
    Validation(Vec<FieldError>),

    /// Selar exige um contêiner em carregamento.
    #[error("Only a container in the loading state can be sealed.")]
    SealRequiresLoading,

    /// Selar exige carga mínima — senão um contêiner quase vazio sairia como
    /// se fosse um carregamento.
    #[error("A container must be at least 10% full to be sealed.")]
    SealBelowMinimumFill,

    /// Despachar exige um contêiner selado. É também o que torna a operação
    /// idempotente no sentido útil: o segundo despacho é recusado.
    #[error("Only a sealed container can be dispatched.")]
    DispatchRequiresSealed,
}

/// Falhas ao embarcar ou desembarcar carga.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// Quantidade nula ou negativa.
    #[error("Quantity must be greater than zero.")]
    InvalidQuantity,

    /// Contêiner selado ou já despachado não recebe carga.
    #[error("Cannot load a sealed or dispatched container.")]
    ContainerClosed,

    /// A carga não cabe.
    #[error("Loading this item would exceed the container capacity.")]
    ExceedsCapacity,

    /// Descarregar exige um contêiner em carregamento.
    #[error("Only a container in the loading state can be unloaded.")]
    UnloadRequiresLoading,

    /// Não há tanto daquele produto embarcado quanto se pediu para tirar.
    #[error("Not enough of this product is loaded to unload the requested quantity.")]
    InsufficientCargo,
}

/// Falhas ao registrar um metadado de sistema (permissão, grupo de marcador).
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// Slug fora do formato exigido.
    #[error("metadado de sistema inválido")]
    Validation(Vec<FieldError>),
}

/// Falhas ao criar um marcador.
#[derive(Debug, thiserror::Error)]
pub enum MarkerError {
    /// Grupo ou valor inválidos.
    #[error("marcador inválido")]
    Validation(Vec<FieldError>),
}
