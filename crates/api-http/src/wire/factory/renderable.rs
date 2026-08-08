//! A resposta já sem tipo, do jeito que a strategy consegue segurar.

use crate::error::api_error::ApiError;
use crate::wire::factory::response_factory::ResponseFactory;

/// Uma resposta que sabe se escrever, sem dizer de que tabela veio.
///
/// ## Por que existem duas traits de resposta
///
/// `Offset<T>` **não faz upcast**: o planus oferece `downcast()` para
/// `Offset<()>` e nada no sentido contrário. Uma strategy que segurasse
/// `&dyn ResponseFactory` não teria como chamar `table()`, porque o tipo
/// associado não sobrevive ao apagamento.
///
/// Daí o par: [`ResponseFactory`] é **tipada**, para o pai aninhar o filho;
/// esta é **apagada**, para a strategy segurar num `Box`. O blanket impl abaixo
/// liga as duas, e nenhuma factory precisa saber que esta trait existe.
///
/// Os bytes são idênticos aos que sairiam pelo caminho tipado:
/// `Offset<T>::ALIGNMENT` é 4 para todo `T`, e o `file_identifier` é `None` nos
/// dois casos.
pub(crate) trait Renderable: Send {
    /// Escreve a tabela no builder e devolve o offset já sem tipo.
    fn write_flatbuffer(
        &self,
        builder: &mut planus::Builder,
    ) -> Result<planus::Offset<()>, ApiError>;

    /// Serializa a tabela como JSON no buffer dado.
    fn write_json(&self, out: &mut Vec<u8>) -> Result<(), ApiError>;
}

impl<F> Renderable for F
where
    F: ResponseFactory + Send,
{
    fn write_flatbuffer(
        &self,
        builder: &mut planus::Builder,
    ) -> Result<planus::Offset<()>, ApiError> {
        use planus::WriteAsOffset as _;

        Ok(self.table()?.prepare(builder).downcast())
    }

    /// Serializa a **tabela do planus**, e não um `serde_json::Value` montado
    /// à mão.
    ///
    /// Não é preferência: o `serde_json` deste lock não tem `preserve_order`,
    /// então o `Map` dele é um `BTreeMap` e os campos sairiam em ordem
    /// alfabética. A tabela sai na ordem de declaração do `.fbs`, que é o que
    /// `swagger/swagger.json` documenta.
    fn write_json(&self, out: &mut Vec<u8>) -> Result<(), ApiError> {
        serde_json::to_writer(out, &self.table()?)
            .map_err(|e| ApiError::unrenderable(format!("falha ao escrever JSON: {e}")))
    }
}
