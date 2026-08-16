//! O contrato de uma versão publicada da API.

use crate::router::route::Route;

/// Uma versão do contrato REST: o número dela, e tudo que ela serve.
///
/// Ver `docs/adr/0010-versions-as-types-with-an-unversioned-alias.md` para o
/// porquê de a versão ser um tipo e não uma constante.
///
/// Uma versão é um **tipo**, e não uma constante, para que a próxima seja um
/// arquivo novo ao lado deste em vez de uma edição no último — que é o que
/// chamar uma versão publicada de congelada tem que significar na prática.
///
/// Uma tabela lista o que aquela versão serve **por inteiro**, e não um delta
/// contra a anterior. Uma rota ausente da `V2Router` simplesmente não é servida
/// sob `/v2`, e essa ausência é como se escreve uma rota que só existiu na v1.
pub(crate) trait VersionedRouter {
    /// O número desta versão, como literal.
    ///
    /// É a fonte única de duas coisas: o prefixo sob o qual as rotas são
    /// montadas (`/v1`) e a ordem em que as versões são classificadas quando o
    /// alias sem versão escolhe um vencedor. Nenhuma das duas está escrita em
    /// outro lugar.
    const VERSION: u16;

    /// Tudo que esta versão publica.
    ///
    /// Não recebe nada: o provider é estático, e cada recurso pede o controller
    /// de que precisa direto a ele.
    ///
    /// A ordem é preservada, e é o que mantém um segmento literal à frente do
    /// `{id}` que também casaria com ele — não porque o axum dependa disso, mas
    /// porque quem lê depende.
    ///
    /// Devolve `Result` porque a maioria dos controllers depende do pool, e
    /// montá-los antes de o boot ter passado os segredos do banco falha. É erro
    /// de boot, e não de requisição: quando a primeira chega, a tabela já
    /// existe.
    fn routes() -> anyhow::Result<Vec<Route>>;
}
