//! A junção de todas as versões publicadas num router só.

use std::collections::HashSet;

use axum::http::Method;
use axum::Router;

use crate::router::intern::v1_router::V1Router;
use crate::router::versioned_router::VersionedRouter;

/// Junta todas as versões publicadas num `Router`.
///
/// Cada versão é montada sob o próprio grupo `/v<n>`, tirado do número que ela
/// mesma declara — o prefixo não tem segunda fonte. Além disso o espaço sem
/// versão também é servido: `/products` alcança a **versão mais nova que a
/// publica**, decidido rota a rota e não globalmente. Uma rota carregada da v1
/// para a v2 responde pela v2 na raiz; uma que só existiu na v1 continua
/// respondendo pela v1.
///
/// A raiz é conveniência, não contrato. O que ela aponta muda no dia em que uma
/// versão nova publicar a mesma rota, então um cliente que quer dizer v1 deve
/// pedir `/v1` — que é o que o `swagger.json` diz a ele no bloco `servers`.
///
/// Struct-namespace: as duas funções são a mesma montagem vista de dois lados.
pub(crate) struct RouterHub;

impl RouterHub {
    /// Monta o router sobre todas as versões que este binário publica.
    ///
    /// **Da mais nova para a mais velha.** É a ordem de que o alias sem versão
    /// depende: a primeira ocorrência de cada par `(verbo, caminho)` é a que
    /// fica, e percorrer nesta ordem é o que a faz ser a mais nova.
    ///
    /// Uma versão nova é uma linha aqui e um arquivo em `intern`.
    pub(crate) fn build() -> anyhow::Result<Router> {
        let mut mounted = Mounted::default();

        Self::mount::<V1Router>(&mut mounted)?;

        Ok(mounted.router)
    }

    /// Monta uma versão sob o prefixo dela, e o que sobrar dela na raiz.
    fn mount<V: VersionedRouter>(mounted: &mut Mounted) -> anyhow::Result<()> {
        anyhow::ensure!(
            mounted.versions.insert(V::VERSION),
            "duas tabelas declaram a versão {}: um número endereça exatamente uma delas",
            V::VERSION
        );

        let prefix = format!("/v{}", V::VERSION);
        let mut versioned = Router::new();

        for route in V::routes()? {
            versioned = versioned.route(route.path, route.handler);
        }

        let mut router = std::mem::take(&mut mounted.router).nest(&prefix, versioned);

        for route in V::routes()? {
            if mounted.published.insert((route.method, route.path)) {
                router = router.route(route.path, route.handler);
            }
        }

        mounted.router = router;

        Ok(())
    }
}

/// O que a montagem acumula enquanto percorre as versões.
///
/// O `published` é o que faz o merge, e é por isso que ele é um conjunto de
/// `(verbo, caminho)` e não uma lista: essa é a identidade que o axum recusa
/// ver duas vezes, então descartar a repetição **antes** de registrar não é
/// economia — é a única forma de registrar, já que o axum entra em pânico ao
/// receber o par repetido em vez de deixar escolher qual fica.
#[derive(Default)]
struct Mounted {
    /// O que já foi montado.
    router: Router,
    /// Os números de versão já vistos, para recusar um repetido.
    versions: HashSet<u16>,
    /// Os pares já publicados na raiz.
    published: HashSet<(Method, &'static str)>,
}
