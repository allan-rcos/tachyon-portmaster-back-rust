//! O contrato de uma consulta de leitura.

/// Uma consulta de leitura, independente de backend.
///
/// Amarra só o tipo de saída. É o que permite ao [`crate::query::query_repository::QueryRepository`] devolver
/// `D::View` sem saber o que `View` é.
pub trait Dql {
    /// O read model que esta consulta produz.
    ///
    /// `Send` porque a View atravessa o `.await` da execução e sai por uma
    /// fronteira de tarefa — o handler que a pediu pode estar em outra thread.
    type View: Send + 'static;

    /// O que distingue esta consulta de outra do mesmo tipo.
    ///
    /// É a identidade da consulta para efeito de cache, e quem a escreve é o
    /// DQL porque é ele quem sabe o que a consulta é. Antes cada serviço do
    /// `app` montava a chave à mão, com um prefixo por serviço e a lembrança de
    /// incluir cada filtro — e um filtro novo que ninguém somasse à chave fazia
    /// duas consultas diferentes lerem a mesma entrada.
    ///
    /// Não sai do SQL: o `QueryBuilder` separa o texto dos valores, e o texto
    /// sozinho é idêntico entre duas páginas do mesmo `SELECT`.
    fn cache_key(&self) -> String;
}
