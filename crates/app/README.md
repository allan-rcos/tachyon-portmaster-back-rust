# `portmaster-app`

A orquestração. Depende de `domain` (as regras) e de `infra` (persistência,
leitura, cache) e é a **única** camada que o `api-http` conhece.

Três coisas acontecem aqui e em nenhum outro lugar: **autorização**,
**transação** e **invalidação de cache**.

```
src/
  provider.rs · register.rs · interno/app_provider.rs
  config/app_secrets.rs   AppSecrets — o que as camadas de baixo pedem
  context/                UserContext, RoleContext — quem está agindo
  error/app_error.rs      AppError — o erro que sobe para a apresentação
  commands/<feature>/     os Command DTOs, um por operação de escrita
  queries/<feature>/      os Query DTOs, um por leitura
  services/               os 10 traits de UseCase — os ports da apresentação
  services/interno/       os 10 impls, privados
  security/               PermissionSlug, PermissionCatalog, RequiresPermission
  cache/                  ReadThrough, CacheKey, Invalidation
  transaction/            Transaction::run
```

## Uma trait por agregado, não por operação

`ProductUseCase` tem `create`, `update`, `delete`, `get`, `list`. Não são cinco
traits. A decisão é deliberada: o `AppProvider` fica com 10 factories em vez de
50, e o grafo monomorfizado continua raso.

## O contexto chega por argumento, sempre

Todo Command e todo Query carrega um `context: UserContext`. Não há estado global,
não há task-local, não há "usuário atual" implícito. O caso de uso confere a
permissão na **primeira linha**:

```rust
self.create_permission.authorize(&command.context)?;
```

A permissão é declarada no **construtor**, não na chamada:

```rust
create_permission: RequiresPermission::new(PermissionSlug::PRODUCT_CREATE),
```

É o que permite ao `PermissionCatalog` ser preenchido no boot a partir do que
cada caso de uso declara exigir — a rota `/metadata/permissions` publica esse
catálogo, e ele não é uma lista mantida à mão.

## Como implementar uma feature nova

Exemplo: arquivar um contêiner.

### 1. O Command

`commands/container/archive_container_command.rs`, um arquivo, um export:

```rust
/// Arquiva um contêiner que já cumpriu o seu ciclo.
pub struct ArchiveContainerCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// O contêiner, em base62.
    pub id: String,
}
```

Campos simples — `String`, `f64`, `Vec<String>`. Nada de `Option` onde a operação
exige o valor: quem trata campo ausente é o `api-http`, e o `TableModule` recusa
com o nome do campo.

### 2. A permissão

`security/permission_slug.rs` ganha a const:

```rust
/// Arquivar um contêiner.
pub const CONTAINER_ARCHIVE: &str = "container:archive";
```

E `security/permission_catalog.rs` a inclui em `ALL`. Esquecer o catálogo faz a
permissão existir e não ser listável — e ninguém consegue concedê-la.

### 3. O método no trait

`services/container_use_case.rs`:

```rust
/// Arquiva um contêiner.
async fn archive(&self, command: ArchiveContainerCommand) -> Result<Box<dyn Container>, AppError>;
```

### 4. O impl

`services/interno/container_use_case_impl.rs`, no formato que os outros seguem:

```rust
async fn archive(&self, command: ArchiveContainerCommand) -> Result<Box<dyn Container>, AppError> {
    self.archive_permission.authorize(&command.context)?;

    let container = Transaction::run(&self.unit_of_work, async {
        let existing = self
            .containers
            .find_by_id(&command.id)
            .await?
            .ok_or_else(|| AppError::not_found("contêiner", &command.id))?;

        let archived = self.container_tm.archive(existing.as_ref())?;
        self.containers.update(archived.as_ref()).await?;

        Ok(archived)
    })
    .await?;

    ReadThrough::invalidate(&self.cache, Invalidation::CONTAINER_WRITE).await?;

    Ok(container)
}
```

Quatro coisas nessa ordem, e a ordem importa:

| Passo | Por quê |
|---|---|
| `authorize` primeiro | Antes de tocar o banco. Um 403 não deve custar uma consulta |
| `Transaction::run` | Tudo que escreve, dentro. Nada de `BEGIN` num repository |
| O `TableModule` constrói | **Nunca** monte um objeto de domínio aqui — a validação teria dois donos |
| Invalidar **fora** da transação | Invalidar dentro e a transação abortar deixaria o cache furado |

### 5. O construtor

O `RequiresPermission` novo entra no `new`, e o `AppProvider` não muda — a trait
do agregado já existia.

## Onde as coisas surpreendem

**A DI estática é frágil de um jeito específico, e há dois testes que a
protegem.** Em `lib.rs` há duas funções `#[cfg(test)]` que não rodam nada e só
precisam **compilar**: elas passam um caso de uso por `tokio::spawn`, que exige
`Send + 'static`. Se um port passar a segurar algo `!Send` através de um
`.await`, o erro apareceria em todo handler do axum — três camadas acima. Os
testes o trazem para cá.

**`AppError::not_found` leva recurso e id, e os dois vão para o corpo.** É
deliberado; o que **não** vai é o slug da permissão negada, que descreveria o
mapa de autorização para quem apanhou um 403.

**A invalidação é por prefixo, e os prefixos moram em `Invalidation`.** Montar
uma chave de cache com um prefixo que a invalidação não conhece faz a entrada
nunca mais sair. Por isso `CacheKey` junta prefixo e construtor no mesmo tipo.

**Este crate reexporta o que o `api-http` precisa e nasce abaixo** — `views`,
`Logger`, os geradores de id, `DomainSecrets`, `InfraSecrets`. Não é conveniência:
é o que mantém uma seta só entre cada par de camadas.
