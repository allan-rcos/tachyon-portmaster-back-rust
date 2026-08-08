# `portmaster-api-http`

A apresentação HTTP. Depende **só** do `app`. Todo o HTTP mora aqui — router,
middlewares, handlers, negociação de conteúdo e JWT — e nenhuma outra camada
conhece token, `FlatBuffers` ou status HTTP.

O crate é `pub(crate)` quase inteiro: o binário é o único consumidor, então a
regra de um export por módulo vale com `pub(crate)` no lugar de `pub`.

```
src/
  main.rs · lib.rs · router.rs      a tabela de rotas e a pilha
  config/                           Env, Secrets, ApiConfig, JwtConfig
  error/api_error.rs                ApiError — a ÚNICA tradução para status HTTP
  session.rs · cookie.rs            a sessão da requisição, e os cookies
  token/token_service.rs            o JWT
  token/refresh_token.rs            o refresh opaco
  middleware/                       5 pares Layer/Service + negotiation
  handlers/                         um módulo por recurso
  handlers/params/                  os filtros de querystring
  wire/wire.rs                      Wire — o que a requisição negociou
  wire/api_response.rs              ApiResponse — o que um handler devolve
  wire/body.rs · json_body.rs       os extractors de corpo
  wire/no_content.rs                o 204
  wire/factory/                     RequestFactory, ResponseFactory, Renderable
  wire/strategy/                    JSON e FlatBuffers, entrada e saída
  wire/dto/<feature>/               DTO + factory, lado a lado
  wire/convert.rs                   as conversões que não são movimento direto
  wire/mod.rs::fbs                  as tabelas geradas pelo planus — NÃO TOCAR
```

## O wire: Abstract Factory + Strategy

Uma requisição chega em `FlatBuffers` ou JSON e a resposta sai em um dos dois.
São **duas responsabilidades separadas**, e é a separação que permite acrescentar
um terceiro formato sem reescrever nada:

- **Strategy** sabe serializar um formato e nada sobre o payload.
- **Factory** sabe os dados de uma mensagem e nada sobre o formato negociado.

O handler não sabe o que foi negociado. Ele entrega os dados a uma factory, e a
strategy conduz.

### Há um `dyn` só, e é na saída

```rust
pub(crate) struct Wire {
    request: MediaType,              // entrada: só o formato
    encode: Arc<dyn EncodeStrategy>, // saída: a strategy
}
```

Assimétrico de propósito. `DecodeStrategy::decode` é **genérico sobre a factory**
e por isso não é object-safe — um `match` no `Body` resolve a escolha de entrada
sem vTable, e o caminho de requisição fica 100% monomorfizado. O `Arc` nasce no
boot (as strategies são ZSTs) e a requisição clona um ponteiro.

### A resposta são duas traits, não uma

`Offset<T>` do planus não faz upcast. Então:

| Trait | Papel |
|---|---|
| `ResponseFactory` | **Tipada**. `fn table() -> Self::Table`. É o que permite ao pai passar a tabela do filho ao planus |
| `Renderable` | **Apagada**, object-safe. É o que a strategy segura. Blanket impl sobre `ResponseFactory` |

Um teste em `wire/dto/auth/login_response_factory.rs` afirma que os bytes são
idênticos pelos dois caminhos.

## Como implementar uma feature nova

Exemplo: `POST /containers/{id}/archive`.

### 1. O DTO e a factory

Em `wire/dto/container/`, dois arquivos lado a lado. Se a operação não tem corpo,
pule para o passo 2.

Resposta reaproveita a factory existente:
`ContainerResponseFactory::of_domain(container.as_ref())`. **Não crie uma segunda
factory para a mesma mensagem** — seria a chance de a resposta do `POST` divergir
da do `GET`.

Se a mensagem é nova, ela precisa existir no `.fbs` primeiro. Os schemas são
**contrato publicado** com os clientes e não se alteram por conveniência nossa.

### 2. O handler

`handlers/container.rs`:

```rust
/// `POST /containers/{id}/archive`
pub(crate) async fn archive(
    &self,
    wire: Wire,
    Path(id): Path<String>,
) -> Result<ApiResponse, ApiError> {
    let context = Session::require_user()?;

    let container = self
        .containers
        .archive(ArchiveContainerCommand { context, id })
        .await
        .map_err(ApiError::of_app)?;

    Ok(ApiResponse::ok(
        wire,
        ContainerResponseFactory::of_domain(container.as_ref()),
    ))
}
```

Um handler faz quatro coisas e nada além: confere sessão, monta o Command,
delega, mapeia de volta. Se apareceu regra de negócio, ela está no lugar errado.

### 3. A rota

`router.rs`, na macro `route!`:

```rust
.route(
    "/containers/{id}/archive",
    post(route!(
        provider,
        container_use_case => ContainerHandlers::archive,
        wire: Wire,
        id: Path<String>,
    )),
)
```

Rota literal antes de rota com parâmetro no mesmo segmento — `/containers/summary`
precisa vir antes de `/containers/{id}`.

## Onde as coisas surpreendem

**O `ApiError` é o único lugar do sistema que conhece status HTTP.**
`ApiError::of_app` é a tradução, com `match` exaustivo — uma variante nova de
`AppError` não passa despercebida como 500. O **401 é o único status que nasce
nesta camada**: é a ausência de sessão, e só quem lê o token sabe disso.

**O corpo de erro é sempre `application/problem+json`,** mesmo quando a
requisição pediu `FlatBuffers`. Um erro pode acontecer **antes** de a negociação
ser resolvida, e aí não existe formato negociado para responder.

**Campo obrigatório ausente é 422, não 400.** Os DTOs têm todos os campos
`Option`; o handler faz `unwrap_or_default()` e o `TableModule` recusa nomeando
**todos** os campos faltantes de uma vez. Exceção deliberada em `/auth/login` e
`/setup`, que seguem em 401 — distinguir "não mandou o campo" de "senha errada"
entregaria ao atacante metade da resposta.

**Corpo ilegível é 400, e não o 404 do PHP.** A suíte Go afirma 404 em cinco
pontos para recurso genuinamente ausente; colidir os dois apagaria a diferença
para qualquer painel ou política de retry.

**O `NegotiationLayer` é load-bearing.** Ele entra imediatamente acima do
`TokenLayer`, e os `.layer()` do axum aplicam-se de baixo para cima. Reordená-lo
faz toda rota com `Wire` responder 500 — deliberado, e é o que o
`FromRequestParts` do `Wire` documenta.

**`wire/mod.rs::fbs` é gerado e tem `#[doc(hidden)]`.** O `build.rs` roda o planus
a cada build e reescreve o arquivo inteiro; um doc escrito ali morre no próximo
`cargo build`. O que a base usa são os DTOs em `wire/dto/`.

**`missing_docs` não cobre nada deste crate,** porque o lint só olha item público
e aqui é tudo `pub(crate)`. A documentação desta camada é sustentada por
disciplina e pelo `clippy::missing_docs_in_private_items`.
