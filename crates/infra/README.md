# `portmaster-infra`

Tudo que fala com o mundo: MariaDB, cache, log, geração de id não-de-domínio.
Depende só do `domain`.

A camada é dividida em **dois lados que não se encontram** — escrita e leitura. É
o CQRS que a API já praticava em PHP, e o que ele compra é que uma listagem
paginada não precisa hidratar um agregado inteiro para exibir quatro colunas.

```
src/
  provider.rs · register.rs · interno/infra_provider.rs
  config/                 InfraSecrets, DatabaseSslMode, SecretString, pool, cache
  repository/             ── ESCRITA: 8 traits, uma por agregado
  repository/mariadb/     os impls SQL
  entity/                 Entity + Row: a tradução linha ↔ model
  entity/codec.rs         Codec — id base62 ↔ i64, enum ↔ índice
  query/                  ── LEITURA
  query/views/            14 Views: o que a leitura devolve, já wire-shaped
  query/dql/              as consultas, uma por arquivo
  query/sql/              Select, Bind, SqlQuery — o construtor de SQL
  query/cursor/           paginação por cursor
  query/params/           os filtros que cada listagem aceita
  database/               UnitOfWork, Scope, pool
  cache/                  ReadCache + os repositories em memória (moka)
  logging/                Logger, LoggerFactory
  id/                     RandomIdGenerator, SortableIdGenerator
  text/search_key.rs      SearchKey — normalização para busca
```

## Os dois lados

| | Escrita | Leitura |
|---|---|---|
| Entrada | `&dyn Model` do `domain` | um DQL com parâmetros |
| Saída | `anyhow::Result<()>` | uma **View** |
| Onde | `repository/` | `query/` |
| Transação | dentro do `UnitOfWork` | fora |

Uma **View** já sai no formato do fio: id em base62, enum como índice, timestamp
em epoch de ms. Ela não é um model — não tem regra, não valida nada, e é isso que
a torna barata.

## Como implementar uma feature nova

### Se é escrita: repository

**1. O método no trait.** `repository/container_repository.rs`:

```rust
/// Marca o contêiner como arquivado.
async fn archive(&self, id: &str) -> anyhow::Result<()>;
```

O trait leva `#[trait_variant::make(Send)]`. Sem ele o futuro não é `Send` e o
handler do axum deixa de compilar — três camadas acima, longe da causa.

**2. O impl.** `repository/mariadb/container_repository.rs`. Todo SQL é
parametrizado; nada de `format!` com valor do usuário.

**3. A tradução, se colunas novas entraram.** `entity/container_entity.rs` e o
`ContainerRow` privado ao lado dele. `Codec::decode_id`/`encode_id` faz a ponte
base62 ↔ `i64` — o banco guarda inteiro, o fio publica base62.

### Se é leitura: DQL + View

**1. A View**, em `query/views/`, um arquivo por View. Campos públicos, sem
método. Contagens são `i64` porque é o que `COUNT(*)` devolve.

**2. O DQL**, em `query/dql/`, um arquivo por consulta. Dois traits:

```rust
impl Dql for GetProductDql {
    type View = Option<ProductViewItem>;   // o que sai
}

impl SqlDql for GetProductDql {
    fn build(&self) -> SqlQuery { … }              // a consulta
    fn read(&self, rows: Vec<MySqlRow>) -> … { … } // a hidratação
}
```

**As colunas são nomeadas, nunca `SELECT *`.** A projeção é o contrato da
hidratação: um `*` faria uma coluna nova entrar na consulta sem ninguém pedir. O
padrão é um `const COLUMNS` no topo do arquivo.

**O filtro de soft-delete é seu.** `deleted_at IS NULL` não vem de graça; sem
ele, um registro removido reaparece na leitura.

**3. O factory.** `query/query_factory.rs` ganha o método, e
`query/interno/mariadb_query_factory.rs` o impl.

### Se é cache

`cache/read_cache.rs` é o contrato; os impls em `cache/interno/` são moka. A
**invalidação** não mora aqui — mora no `app`, em `cache/invalidation.rs`, porque
quem sabe que uma escrita aconteceu é o caso de uso.

## Onde as coisas surpreendem

**Todo horário é UTC, e isso é fixado no pool.** As colunas são `DATETIME`, que o
MariaDB guarda sem converter, e o fuso da sessão decide o que `CURRENT_TIMESTAMP`
vale no INSERT. `database/pool.rs` fixa `+00:00` **depois** do parse da URI —
antes, uma `?timezone=` na URI o sobrescreveria. `chrono::Local` está banido por
lint.

**`SearchKey` normaliza o texto de busca, e a coluna `search_*` guarda o
resultado.** Buscar sem passar por ela produz zero resultados para acentuação
diferente.

**A paginação por cursor satura em vez de estourar.** `page.saturating_sub(1)
.saturating_mul(limit)` — a versão anterior estourava `u32` e a página 0 virava
offset gigante.

**O `UnitOfWork` é quem abre transação, mas quem a demarca é o `app`.** Um
repository nunca faz `BEGIN`.
