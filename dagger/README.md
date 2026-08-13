# dagger/

**As verificações e o empacotamento da API em Rust, sem um script de build no repositório.**

Esta API é uma **segunda implementação da mesma API** que `back-php` serve, e o contrato de artefato é deliberadamente idêntico: os mesmos nomes de asset, o mesmo par de tarballs, o mesmo `.sha256` ao lado. A infraestrutura escolhe qual das duas implanta sem saber que a linguagem mudou.

O que difere é o conteúdo. Aqui o tarball da API é **um binário estático musl**; no PHP é `src/` mais `vendor/` minificados. Quem precisa saber a diferença é a role do Ansible que desempacota — ver `ansible/roles/rust-api` na infraestrutura.

-----

## 🗺️ O diretório

```
dagger/
├── dagger.json     SDK Go e as dependências locais
├── main.go         só o tipo e a explicação do diretório
├── ci.go           fmt → clippy → lint-exports → check-doc → test
├── fmt.go clippy.go lintexports.go doc.go test.go
├── dist.go version.go
├── fbsgo.go fbscheck.go
├── integration.go  a suíte Go, com o daemon Docker junto
└── modules/
    ├── toolchain/  em que Rust este repositório constrói
    ├── artifact/   os dois tarballs
    └── codegen/    os bindings Go dos schemas
```

Um arquivo por comando: para mexer no que `dagger call clippy` faz, abra `clippy.go`.

-----

## 🚀 O que você vai rodar

De dentro de `dagger/`, e nenhum deles pede Rust instalado:

```bash
dagger call ci                    # tudo que tem de estar verde antes de um merge
dagger call fmt                   # cargo fmt --all --check
dagger call clippy                # -D warnings; sem isso o lint não vale
dagger call test
dagger call lint-exports          # o xtask do próprio projeto
dagger call doc export --path ../docs/rustdoc
dagger call dist --version 0.1.0 export --path ../dist
dagger call check-fbs-go
dagger call integration-test
```

-----

## 📦 O que era `scripts/`

O diretório não existe mais. Quatro scripts foram reimplementados, e cada porte foi conferido contra a saída do original **antes** de o original ser apagado.

| Era | Virou | Prova |
|---|---|---|
| `build-dist.sh` | `modules/artifact` | Binário **byte a byte idêntico**: 8.339.264 bytes, mesmo sha256, mesma estrutura de tar. As migrations idênticas |
| `generate-docs.sh` | `doc.go` | Mesmo `RUSTDOCFLAGS=-D warnings`, mesmo redirect para `portmaster_domain` |
| `generate-flatbuffers-go.sh` | `modules/codegen` | `check-fbs-go` verde contra os bindings commitados |
| `integration-test.sh` | `integration.go` | O mesmo `go test ./... -count=1 -timeout 20m` |

> **A igualdade byte a byte do binário custou uma descoberta.** A primeira versão do módulo desviava `CARGO_HOME` para o volume de cache, e o artefato saía **4096 bytes diferente** do que o script produzia — mesma versão de `rustc`, mesmo código, binário diferente. O caminho do registry acaba embutido em mensagens de panic e metadados que o `strip` não remove. Cachear só o `registry` **dentro** do `CARGO_HOME` padrão da imagem resolveu, e é por isso que `modules/toolchain` monta em `/usr/local/cargo/registry` em vez de trocar a variável.

-----

## 📋 Os métodos

Doze funções. Nenhuma pede Rust instalado; `dagger call <nome> --help` lista os argumentos.

| Comando | Devolve | Arquivo | Para quê |
|---|---|---|---|
| `ci` | `String` | `ci.go` | `fmt` → `clippy` → `lint-exports` → `check-doc` → `test` |
| `fmt` | `String` | `fmt.go` | `cargo fmt --all --check` — confere, não reescreve |
| `clippy` | `String` | `clippy.go` | O linter com `-D warnings` |
| `lint-exports` | `String` | `lintexports.go` | O `xtask` do próprio projeto |
| `test` | `String` | `test.go` | `cargo test --workspace` |
| `check-doc` | `String` | `doc.go` | Constrói a doc e falha em link quebrado |
| `doc` | `Directory` | `doc.go` | O mesmo, devolvendo `docs/rustdoc` |
| `dist` | `Directory` | `dist.go` | Os dois tarballs de produção |
| `version` | `String` | `version.go` | A versão de `[workspace.package]` |
| `generate-fbs-go` | `Directory` | `fbsgo.go` | Regera os bindings Go da suíte |
| `check-fbs-go` | `String` | `fbscheck.go` | Falha se os commitados estiverem defasados |
| `integration-test` | `String` | `integration.go` | A suíte Go, com o daemon Docker junto |

**`doc` e `check-doc` são o mesmo trabalho com saídas diferentes.** O `ci` chama `check-doc` porque ali interessa o veredito; `doc` devolve o HTML para publicar:

```bash
dagger call doc  export --path ../docs/rustdoc
dagger call dist --version 0.1.0 export --path ../dist
dagger call generate-fbs-go export --path ../tests/integration/internal/fbs
```

### Argumentos

Quase todas recebem só `--source`, com `defaultPath` para a raiz — na prática você não passa nada. As exceções:

| Comando | Argumento | Padrão | |
|---|---|---|---|
| `dist` | `--version` | de `Cargo.toml` | Sobrepor para uma beta |
| `version` | `--cargo-toml` | `/Cargo.toml` | |
| `integration-test` | `--args` | — | Repassado ao `go test`: `--args -run,TestYardStory` |
| `integration-test` | `--pool-size` | GOMAXPROCS | Ambientes {API + banco} em paralelo |

-----

## ➕ Como acrescentar um método

1. **Decida onde.** Compõe outras? Arquivo na raiz de `dagger/`. Constrói ambiente ou implementa lógica? Um módulo — `toolchain`, `artifact` ou `codegen`. A R2 diz que a raiz não tem `Container.From()`, e `integration.go` é a exceção documentada.

2. **Um arquivo, com o nome do comando.**

3. **Escreva a função.** A primeira linha do comentário vira a descrição:

   ```go
   package main

   import (
   	"context"
   	"dagger/back-rust/internal/dagger"
   )

   // Audit roda o cargo-deny sobre as licenças e avisos de segurança.
   func (m *BackRust) Audit(
   	ctx context.Context,
   	// +defaultPath="/"
   	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
   	source *dagger.Directory,
   ) (string, error) {
   	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
   		WithExec([]string{"cargo", "install", "cargo-deny", "--locked"}).
   		WithExec([]string{"cargo", "deny", "check"}).
   		Stdout(ctx)
   }
   ```

   Note o `+ignore` deste repositório: `target` no lugar de `vendor`, e `tmp` — ele é diferente do dos outros e tem de ser igual ao das funções vizinhas.

4. **`dagger develop`** no módulo alterado, depois na raiz.

5. **`dagger functions`** para confirmar.

### As armadilhas de quem escreve um método novo

> **Nunca termine um arquivo em `_test.go`.** O sufixo é reservado pelo Go, o arquivo fica fora do build, e o único sintoma é `unknown command`.

> **Toda função que compila precisa passar por `toolchain.Dev()`.** É lá que o `rustup target add` roda DEPOIS de o código estar montado — o `rust-toolchain.toml` só é honrado a partir do diretório de trabalho, e um alvo adicionado antes vai para a toolchain errada. O erro é `can't find crate for core`, que não menciona toolchain.

> **Não desvie o `CARGO_HOME`.** O `Dev()` monta o cache dentro do `CARGO_HOME` padrão da imagem justamente porque o caminho do registry acaba embutido no binário: desviá-lo muda o artefato em 4096 bytes, com o mesmo `rustc` e o mesmo código.

> **`Api` vira `API` no binding gerado**, e `Json` vira `JSON`.

### E se a função substitui um script

Reimplementar, não envelopar — e **provar equivalência antes de apagar**. As provas dos quatro scripts que saíram daqui estão na seção acima; a do `build-dist.sh` chegou a binário byte a byte idêntico.

-----

## ⚠️ As armadilhas

> **O `rustup target add` tem de rodar DEPOIS de montar o código.** O repositório traz um `rust-toolchain.toml`, e o rustup o honra a partir do diretório de trabalho: assim que o cargo roda em `/work`, a toolchain ativa é a que aquele arquivo pede, não a que veio na imagem. Um `rustup target add` feito antes instala o std na toolchain errada, e o `cargo build --target` falha com **`can't find crate for core`** — uma mensagem que fala de crate e não diz nada sobre toolchain.

> **A ordem dos mounts decide se o cache existe.** O diretório do código entra antes do cache de `target/`. Invertido, o mount de `/work` cobre o de `/work/target` e o cache não vale nada — sem erro nenhum, só recompilação integral a cada execução.

> **O tar não tem diretório versionado no topo.** `tar -C .stage/api -c .` põe o conteúdo na raiz. A guarda de raiz única do Dockerfile da infraestrutura **passa** — `.` conta como uma raiz — então ela nunca vai avisar que o artefato do formato errado chegou. Quem protege é a role certa para a variante certa.

> **O `swagger` é o mesmo submódulo do repositório em PHP.** Um `.fbs` novo obriga a regerar bindings nos dois, e a divergência aparece como incompatibilidade de wire em runtime, não como erro de build.

-----

## 📚 Relacionado

* [`../../dagger/README.md`](../../dagger/README.md) — as regras de arquitetura e como as variantes são escolhidas.
* [`../../back-php/dagger/README.md`](../../back-php/dagger/README.md) — a outra implementação da mesma API.
* [`../../ansible/roles/rust-api`](../../ansible/roles/rust-api) — a role que instala este binário.
