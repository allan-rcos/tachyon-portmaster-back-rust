//! O contrato de uma consulta de leitura que fala SQL.

use sqlx::mysql::{MySql, MySqlRow};
use sqlx::QueryBuilder;

/// A face SQL de uma consulta — a que vale hoje.
///
/// Um backend novo ganha a sua própria face (`MongoDql`, com `filter`/`options`)
/// e o repositório correspondente passa a consumi-la. Nada disso alcança a View.
pub trait SqlDql: super::Dql + Send {
    /// Compila a consulta.
    ///
    /// Chamado uma vez por execução, então o DQL monta o SQL a partir dos
    /// filtros que recebeu em vez de guardar um texto pronto.
    ///
    /// O construtor é o do próprio `sqlx`: `push` para texto literal,
    /// `push_bind` para valor. Não há construtor de `SELECT` nosso no caminho —
    /// ele era uma segunda linguagem para manter, e o `push_bind` já é o que
    /// impedia a interpolação de valor que ele existia para impedir.
    fn build(&self) -> QueryBuilder<MySql>;

    /// Transforma as linhas na View.
    ///
    /// É o único lugar que conhece o tipo concreto de saída — o que deixa o
    /// repositório genérico.
    ///
    /// Falha quando uma linha não corresponde ao schema: um índice de enum fora
    /// da faixa, por exemplo. A alternativa seria escolher uma variante por
    /// aproximação, e uma View que reporta `Class1Explosives` porque o valor
    /// gravado não bateu com nada estaria afirmando que a carga é explosiva.
    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View>;
}
