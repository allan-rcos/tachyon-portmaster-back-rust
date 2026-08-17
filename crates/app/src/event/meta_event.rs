//! O que um caso de uso tem a dizer a um middleware.

/// Qualquer coisa que um service precise comunicar a um middleware.
///
/// É essa a regra de admissão, e ela é ampla de propósito: um caso de uso sabe
/// algo sobre como produziu a resposta, um middleware precisa desse algo para
/// decidir o que fazer com ela, e não existe caminho entre os dois. O retorno do
/// caso de uso é a View — o que o cliente pediu —, e pendurar nele um segundo
/// valor que só a borda entende contaminaria a assinatura de todo mundo no meio.
/// Um `MetaEvent` é esse segundo canal.
///
/// Meta e não domínio: nada aqui descreve o que aconteceu no pátio. O que um
/// evento descreve é o **caminho** que a requisição tomou por dentro — se a
/// resposta saiu do cache, se uma regra caiu num ramo degradado, se algo foi
/// servido de uma fonte alternativa. Se o cliente precisa do dado como
/// informação de negócio, ele pertence à View e não a este enum.
///
/// Acrescentar um evento é acrescentar uma variante aqui e um `emit` no caso de
/// uso que a conhece. Nenhuma assinatura entre os dois muda, e nenhum middleware
/// que não se importe fica sabendo.
///
/// ## Um bit por variante
///
/// O discriminante é o índice do bit na máscara que a pilha guarda, e é por isso
/// que o enum é `#[repr(u8)]`: a representação da variante é o dado, não um
/// detalhe do compilador. Quem faz a conta é a implementação da pilha, em
/// `event/intern/`, e alargar a máscara é assunto exclusivamente dela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MetaEvent {
    /// A View devolvida saiu do cache de leitura, e não do banco.
    ViewCacheHit = 0,
}
