//! Como um repositório em memória grava e lê.

use std::sync::Arc;
use std::time::Duration;

/// Um armazenamento em memória, sobre bytes.
///
/// A chave tem duas partes porque um store guarda mais de um recorte: o
/// **grupo** é o namespace (a permissão, o grupo de marcador, o recurso cuja
/// listagem foi cacheada) e a **chave** identifica a entrada dentro dele. É o
/// que faz `clear_group` ser um recorte exato em vez de uma varredura por
/// prefixo, que casaria `container:` com `container_summary:`.
///
/// Fala bytes, e não objeto de domínio, pela mesma razão que o `MySqlTransaction`
/// fala linha: quem traduz é o repositório. Aqui a tradução é serialização em
/// vez de coluna, e é a única diferença entre um repositório destes e um do
/// `MariaDB`.

#[trait_variant::make(Send)]
pub(crate) trait MemoryStore {
    /// O valor da entrada, se ela existir e não tiver expirado.
    ///
    /// Dentro de um escopo, o que a própria tarefa escreveu vem antes do que o
    /// cache tem: sem isso, gravar e ler na mesma tarefa devolveria o valor
    /// anterior, e nenhum repositório do `MariaDB` se comporta assim.
    async fn get(&self, group: &str, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>>;

    /// Grava a entrada, com o prazo dela.
    ///
    /// `ttl` ausente significa "vive enquanto o processo viver" — é o caso do
    /// catálogo de permissões, que o boot preenche e ninguém mais altera. O
    /// prazo viaja com o valor porque cada marcador vence no seu próprio tempo,
    /// não num vencimento global.
    async fn put(
        &self,
        group: &str,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> anyhow::Result<()>;

    /// Descarta o grupo inteiro.
    ///
    /// Sem varredura por prefixo: o grupo já é o recorte, e comparar prefixo
    /// casaria `container` com `container_summary`.
    async fn clear_group(&self, group: &str) -> anyhow::Result<()>;

    /// As chaves vivas de um grupo, em ordem.
    ///
    /// Ordenado porque a iteração do Moka não tem ordem definida, e uma listagem
    /// que muda de ordem a cada chamada é ruim de ler e pior de testar.
    async fn keys(&self, group: &str) -> anyhow::Result<Vec<String>>;
}
