# `crates/` — as quatro camadas, e como uma feature atravessa

Quatro crates, uma seta entre cada par, nenhuma volta:

```
api-http  ──▶  app  ──▶  infra  ──▶  domain
                 └──────────────────────▶
```

`api-http` conhece **só** o `app`. O `app` conhece `infra` e `domain`. A `infra`
conhece o `domain`. O `domain` não conhece ninguém. Uma dependência a mais nesse
grafo vale para sempre, então cada uma precisou de argumento — é por isso que o
`app` reexporta `views`, `Logger` e os geradores de id: sem isso o `api-http`
teria que declarar `portmaster-infra` para nomear um tipo que só repassa.

| Crate | Do que é dono | README |
|---|---|---|
| `domain` | Regras de negócio, models, `TableModules`, hashing, id | [domain/README.md](domain/README.md) |
| `infra` | Banco, repositories, `UnitOfWork`, cache, leitura (DQL/Views), log | [infra/README.md](infra/README.md) |
| `app` | Casos de uso, Commands, Queries, autorização, transação | [app/README.md](app/README.md) |
| `api-http` | Router, handlers, middlewares, negociação de conteúdo, JWT | [api-http/README.md](api-http/README.md) |

## As duas regras que valem em toda parte

### Um export por módulo

Um arquivo `.rs` expõe **no máximo um** item — `pub` nas bibliotecas,
`pub(crate)` no `api-http`. O nome do arquivo declara o que ele exporta:
`services/container_use_case.rs` exporta `ContainerUseCase`;
`services/interno/container_use_case_impl.rs` exporta o impl, privado.

Não contam como export: `mod foo;`, `use`/`pub use`, blocos `impl`,
`#[cfg(test)] mod tests` e `pub(super)`. `cargo xtask lint-exports` trava a regra
no CI, sobre os 424 arquivos.

Quando a regra empurraria para arquivos de três linhas, use a **struct-namespace**
— um tipo sem estado com funções associadas. `Base62`, `SearchKey`, `Codec`,
`Row`, `PermissionSlug`, `CacheKey`, `Convert`, `Env` e `RefreshToken` são todos
isso.

### A injeção é 100% estática

Não há container, não há composition root e não há `dyn` no wiring. Cada camada
tem um `provider.rs` (o trait dos factories) e um `register.rs` (o construtor), e
cada factory devolve `impl Trait`:

```rust
fn product_use_case(&self) -> impl ProductUseCase + Send + Sync;
```

O consumidor recebe o **contrato** e o compilador monomorfiza o grafo inteiro. O
efeito prático é que um serviço que não pode ser construído é erro de compilação,
e não surpresa no boot.

O preço é que os tipos concretos são **innomeáveis** — só existem depois da
monomorfização. Por isso tudo do router para baixo é genérico: os handlers sobre
os casos de uso, a função `router` sobre o provider.

Há **dois** `dyn` no sistema inteiro, os dois documentados onde vivem:

1. `Box<dyn User>` e irmãos — o que um caso de uso de escrita devolve. Read-only
   e mapeado direto para o fio.
2. `Arc<dyn EncodeStrategy>` — a strategy de saída da negociação de conteúdo,
   escolhida por requisição.

## Como uma feature nova atravessa

O caminho é sempre o mesmo, de baixo para cima. Tomando "arquivar um contêiner"
como exemplo:

| # | Camada | O que nasce |
|---|---|---|
| 1 | `domain` | A regra: o `ContainerTM` ganha o método que valida a transição e devolve o model, ou o erro tipado |
| 2 | `infra` | A persistência: método no `ContainerRepository` + o impl em `repository/mariadb/` |
| 3 | `app` | O Command (`ArchiveContainerCommand`), o método no `ContainerUseCase` + impl em `services/interno/`, a permissão em `PermissionSlug` |
| 4 | `api-http` | O DTO+factory em `wire/dto/container/`, o handler, a rota |

Cada README detalha os arquivos exatos da sua camada. **Comece sempre pelo
`domain`**: se a regra não existe lá, ela vai acabar escrita num handler.

## Documentação

Prosa em **português**, e o projeto não usa as seções `# Errors`/`# Panics` do
rustdoc — que são convenção em inglês. O que cada erro significa está escrito na
prosa do próprio item.

Comentário de código (`//`) tem **no máximo uma linha**. Racional extenso sobe
para o `///` do item; função grande demais para caber sem ele ganha um helper
privado, e a explicação vai na doc do helper. O motivo é simples: o que está no
corpo não aparece no rustdoc, e quem lê a referência renderizada nunca vê.

```bash
dagger call doc     # → docs/rustdoc, servido pelo GitHub Pages
```

Link de doc quebrado **derruba o build** (`[workspace.lints.rustdoc]` em `deny`),
porque um `[`Coisa`]` que não resolve vira texto simples em silêncio — a falha é
invisível exatamente na página que ela estraga.

## O portão

Tudo isto precisa passar antes de commitar:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run --package xtask --locked -- lint-exports
cargo doc --workspace --no-deps --document-private-items
cargo test --workspace
cargo deny check bans
```

E a paridade de fio, que é o juiz final — a suíte Go fala `FlatBuffers` com a API
de verdade, sobre containers de verdade:

```bash
dagger call integration-test
```
