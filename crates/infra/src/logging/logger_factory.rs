//! O contrato da fábrica de loggers.

use crate::logging::Logger;

/// Cria loggers nomeados.
///
/// O nome identifica a origem — `auth`, `http`, `container` — e vira campo em
/// toda linha que aquele logger emitir. É por isso que o nome é **propriedade
/// do logger** e não um argumento de cada chamada: quem cria já sabe quem é, e
/// quem escreve a linha não deveria precisar repetir.
///
/// O logger sai por **tipo associado** e não por `Box<dyn Logger>`: a impl é
/// única, e uma indireção que nunca aponta para dois lugares diferentes só
/// custa. Associado e não `impl Logger` porque quem o consome precisa às vezes
/// **nomear** o tipo — o `Layer` do `tower` declara o serviço que produz num
/// tipo associado, e ali um `impl Trait` não tem nome para dar.
pub trait LoggerFactory: Clone + Send + Sync + 'static {
    /// O logger que esta fábrica produz.
    type Instance: Logger;

    /// Um logger para o componente indicado.
    fn create(&self, name: &str) -> Self::Instance;
}
