# `portmaster-domain`

As regras de negócio, e nada mais. Este crate **não depende de nenhum outro** —
nem do `infra`, nem de banco, nem de HTTP. É por isso que ele é testável sem
preparar ambiente e é onde uma regra vai parar quando você não sabe onde pôr.

```
src/
  provider.rs            trait DomainProvider — os 9 factories
  register.rs            fn register — o construtor da camada
  config/                DomainSecrets (cluster_id, server_id do Snowflake)
  enums/                 ContainerStatus, RiskClass, TelemetryEvent
  error/                 um erro tipado por agregado + FieldError
  error/interno/         Validation, o acumulador de campos recusados
  models/                as traits de model (Product, Container, User, …)
  models/interno/        os models concretos, privados
  table_modules/         as traits de TableModule — onde a regra mora
  table_modules/interno/ os impls, privados
  security/              PasswordHasher, IndexHasher
  security/interno/      Argon2 e xxHash
  id/                    Base62, IntIdGenerator
  id/interno/            o gerador Snowflake
```

## O que é um `TableModule`

É o objeto que **constrói e altera** um agregado, e o único lugar onde a regra
daquele agregado existe. Um `ProductTM` sabe que densidade precisa ser maior que
zero; ninguém mais sabe, e ninguém mais deveria perguntar.

Duas características valem entender antes de escrever um:

**Ele devolve `Result<Box<dyn Model>, XError>`.** O erro é tipado por agregado —
`ProductError`, `ContainerError` — e não uma string. Quem traduz aquilo em status
HTTP é o `api-http`, num lugar só.

**Ele acumula os campos recusados, não para no primeiro.** É o que faz um 422
nomear tudo que faltou de uma vez, em vez de devolver um problema por
requisição. O acumulador é `error/interno/validation.rs`.

## Como implementar uma feature nova

Exemplo: arquivar um contêiner.

### 1. O erro, se a regra puder falhar de um jeito novo

`error/container_error.rs` ganha a variante. Se o agregado ainda não tem erro
próprio, o arquivo é novo — um export por módulo.

```rust
/// Um contêiner em trânsito não pode ser arquivado.
EmTransito,
```

### 2. O método no trait

`table_modules/container_tm.rs`:

```rust
/// Produz o contêiner arquivado, a partir de um que possa sê-lo.
///
/// Um contêiner em trânsito não pode: a carga ainda é responsabilidade do
/// pátio até a entrega, e arquivá-lo apagaria o único registro de onde ela
/// está.
fn archive(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;
```

O doc do método é onde o **porquê** da regra fica. Não é comentário no corpo do
impl: ali ele não aparece no rustdoc, e quem lê a referência nunca vê.

### 3. O impl

`table_modules/interno/container_tm_impl.rs`. Privado, e é a razão de o
`interno/` existir: quem consome recebe o contrato, nunca o tipo concreto.

### 4. Model e enum, se o estado for novo

Estado novo entra em `enums/container_status.rs`. **Atenção:** os índices desse
enum são os mesmos do enum do `.fbs`, e o `api-http` converte pelo número — uma
variante inserida no meio muda o significado de dados já gravados.

### 5. O teste

No mesmo arquivo, `#[cfg(test)] mod tests`. A camada não tem I/O, então o teste é
uma chamada e um `assert` — sem fixture, sem container, sem banco.

```rust
#[test]
fn um_conteiner_em_transito_nao_arquiva() {
    // A carga ainda é do pátio até a entrega.
    let erro = tm().archive(&em_transito()).expect_err("deveria recusar");
    assert!(matches!(erro, ContainerError::EmTransito));
}
```

## O que **não** entra aqui

| Coisa | Onde vai |
|---|---|
| SQL, qualquer SQL | `infra` |
| Ordem de operações, transação | `app` |
| Autorização, permissão | `app` |
| Status HTTP | `api-http` |
| Leitura para listagem | `infra`, como DQL + View |

A regra prática: se precisou de `async`, provavelmente não é desta camada. O
`domain` é síncrono de ponta a ponta — não há I/O para esperar.

## Onde as coisas surpreendem

**`Base62` é struct-namespace, não módulo de funções soltas.** `Base62::encode` e
`Base62::decode`. Cuidado ao chamar: existe um crate `base62` no workspace, e
`base62::decode` resolve para ele.

**O `IndexHasher` é rápido de propósito, e o `PasswordHasher` é lento de
propósito.** Trocar um pelo outro é o tipo de erro que não aparece em teste
nenhum. Está escrito no doc de `security/mod.rs`.

**Ids são Snowflake, e o `DomainSecrets` carrega quem é este servidor na
composição.** Dois processos com o mesmo `server_id` geram ids que colidem.
