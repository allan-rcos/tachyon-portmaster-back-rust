//! O contrato de uma versão publicada da API.

use crate::bootstrap::provider::ApiProvider;
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
    /// Recebe o provider e não os controllers: cada recurso sabe de qual
    /// controller precisa, e uma versão nova que sirva os mesmos recursos não
    /// deveria ter de repetir a lista de dez fábricas.
    ///
    /// A ordem é preservada, e é o que mantém um segmento literal à frente do
    /// `{id}` que também casaria com ele — não porque o axum dependa disso, mas
    /// porque quem lê depende.
    fn routes<P: ApiProvider>(provider: &P) -> Vec<Route>;
}
