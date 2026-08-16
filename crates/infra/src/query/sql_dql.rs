//! O contrato de uma consulta de leitura que fala SQL.

use mysql_async::{Params, Row};

/// A face SQL de uma consulta — a que vale hoje.
///
/// Um backend novo ganha a sua própria face (`MongoDql`, com `filter`/`options`)
/// e o repositório correspondente passa a consumi-la. Nada disso alcança a View.
pub trait SqlDql: super::Dql + Send {
    /// Compila a consulta: o texto e os valores que ele nomeia.
    ///
    /// Chamado uma vez por execução, então o DQL monta o SQL a partir dos
    /// filtros que recebeu em vez de guardar um texto pronto.
    ///
    /// Os valores vão por parâmetro **nomeado**, nunca interpolados no texto.
    /// Nomeado e não posicional porque um mesmo valor costuma aparecer duas
    /// vezes na mesma consulta — o filtro da página e o filtro da contagem — e
    /// com nome ele é declarado uma vez só, sem depender de a ordem das ligações
    /// acompanhar a ordem em que o texto foi montado.
    fn build(&self) -> (String, Params);

    /// Transforma as linhas na View.
    ///
    /// É o único lugar que conhece o tipo concreto de saída — o que deixa o
    /// repositório genérico.
    ///
    /// Falha quando uma linha não corresponde ao schema: um índice de enum fora
    /// da faixa, por exemplo. A alternativa seria escolher uma variante por
    /// aproximação, e uma View que reporta `Class1Explosives` porque o valor
    /// gravado não bateu com nada estaria afirmando que a carga é explosiva.
    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View>;
}
