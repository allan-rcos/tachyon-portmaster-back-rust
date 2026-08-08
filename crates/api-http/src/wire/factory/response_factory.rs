//! O contrato de quem monta um corpo de resposta.

use serde::Serialize;

use crate::error::api_error::ApiError;

/// Monta a tabela que vai sair no fio.
///
/// **Tipada de propósito**, e é isso que a distingue da
/// [`Renderable`](super::renderable::Renderable): conhecer `Self::Table` é o que
/// permite a uma factory pai embutir o resultado de uma factory filha —
/// `LoginResponse` carrega um `User`, e o campo espera o tipo, não bytes.
///
/// ## O planus já escreve a coreografia de builder
///
/// O impl de `WriteAsOffset` que o planus gera para cada tabela é, linha por
/// linha, o `createXVector` + `createX` que o PHP escreve à mão. A factory
/// produz a tabela *owned* e deixa o planus aninhar — reimplementar isso seria
/// reescrever o código gerado.
pub(crate) trait ResponseFactory {
    /// A tabela gerada pelo planus que esta factory produz.
    type Table: Serialize + planus::WriteAsOffset<Self::Table>;

    /// Monta a tabela.
    ///
    /// Falível porque alguns campos podem não caber no que o `.fbs` declara —
    /// um epoch que não vira RFC 3339, por exemplo. A maioria das factories
    /// devolve `Ok` sempre.
    fn table(&self) -> Result<Self::Table, ApiError>;
}
