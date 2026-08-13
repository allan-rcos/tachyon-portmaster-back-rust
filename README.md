# Portmaster

**API de pátio de contêineres construída em Rust sobre axum e tokio, falando FlatBuffers binário sobre HTTP.**

Produtos são catalogados, contêineres são registrados, carga é embarcada e desembarcada contra um manifesto, e o contêiner é selado e despachado quando atinge sua capacidade. Um processo, muitas threads, sem servidor web na frente: o binário *é* o servidor, com o pool de conexões e os caches vivos entre requisições.

O projeto é uma **base de ecossistema para APIs de alto desempenho** — arquitetura em quatro camadas com dependências de mão única, injeção de dependências 100% estática (sem `dyn`, sem container, sem reflexão), autorização declarada pelo próprio caso de uso, e um formato de fio binário gerado a partir de schemas versionados.

[![CI](https://img.shields.io/github/actions/workflow/status/allan-rcos/tachyon-portmaster-back-php/ci.yml?branch=main&style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/allan-rcos/tachyon-portmaster-back-php/actions/workflows/ci.yml)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-1F1F1F?style=for-the-badge&logo=rust&logoColor=white)
![axum](https://img.shields.io/badge/axum-1F6FEB?style=for-the-badge&logo=rust&logoColor=white)
![FlatBuffers](https://img.shields.io/badge/FlatBuffers-4285F4?style=for-the-badge&logo=google&logoColor=white)
![MariaDB](https://img.shields.io/badge/MariaDB-003545?style=for-the-badge&logo=mariadb&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Go](https://img.shields.io/badge/go-%2300ADD8.svg?style=for-the-badge&logo=go&logoColor=white)
![JWT](https://img.shields.io/badge/JWT-black?style=for-the-badge&logo=jsonwebtokens&logoColor=white)
![Clippy](https://img.shields.io/badge/clippy_--D_warnings-DEA584?style=for-the-badge&logo=rust&logoColor=black)
![Testcontainers](https://img.shields.io/badge/Testcontainers-291A3F?style=for-the-badge&logo=docker&logoColor=white)
![OpenAPI](https://img.shields.io/badge/OpenAPI-6BA539?style=for-the-badge&logo=openapiinitiative&logoColor=white)
![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-2088FF?style=for-the-badge&logo=githubactions&logoColor=white)
![License](https://img.shields.io/badge/MIT-green?style=for-the-badge)

-----

## ✨ Destaques

* **Um processo, sem fork.** tokio dimensiona o próprio pool de threads, e tudo que é caro — pool de conexões, caches — nasce uma vez no `register` e é compartilhado por `Arc`. Os registros de runtime que antes precisavam de tabelas no banco para atravessar quatro workers agora são mapas em memória, porque não há mais fronteira a atravessar ([ADR 0009](docs/adr/0009-runtime-registries-in-process.md)).
* **Injeção de dependências 100% estática.** Cada provider entrega `impl Trait` — RPITIT, resolvido na monomorfização. Não há `dyn` no wiring, não há container, não há resolução em runtime: o grafo inteiro é um tipo que o compilador conhece. O `dyn` aparece em dois lugares só, ambos de borda: o objeto de domínio que uma escrita devolve, e o futuro de uma requisição dentro de um middleware.
* **FlatBuffers gerado em Rust.** Os tipos de wire saem dos `.fbs` de `swagger/` durante o build, pelo `planus` — **sem `flatc`**, que não existiria na imagem final e tornaria o build refém de uma ferramenta externa. Os mesmos tipos serializam JSON pelo serde, negociado por `Accept`/`Content-Type`.
* **Quatro camadas, dependências de mão única.** `api-http → app → infra → domain`. O `domain` não depende de nada.
* **Erros como valores, e o status só na borda.** `thiserror` no domínio e no app, `anyhow` na infra, e um único ponto de tradução para HTTP no `api-http`. Nenhuma camada de baixo sabe o que é um número de status — o mesmo erro serve a uma saída que não seja HTTP.
* **Autorização junto do caso de uso.** Cada caso de uso protegido declara sua permissão no construtor e a exige na primeira linha, contra o contexto que veio no Command — nunca de um estado global. O catálogo é o próprio código: `POST /setup` concede ao primeiro papel toda permissão registrada, então uma permissão nova nasce concedida sem lista para manter.
* **Regras em um só lugar.** Validação vive nos *table modules* do domínio, que se recusam a construir um modelo inválido e acumulam **todos** os campos recusados em vez de parar no primeiro. Handler não valida, repositório não valida.
* **Ids opacos na borda.** Snowflake para entidade, NanoID para o refresh token, xid para o `request_id`. `BIGINT` no banco, Base62 na API — por isso as rotas casam `[A-Za-z0-9]+`, nunca `\d+`.
* **Leituras e escritas assimétricas.** Escrita atravessa `Command → UseCase → TableModule → Repository` dentro de uma transação; leitura atravessa `Query → DQL → View` e devolve exatamente as colunas do endpoint, paginada por cursor, sem reconstituir modelo de domínio.
* **Qualidade verificada.** `clippy -D warnings` sobre todos os alvos, testes unitários em cada camada, e uma suíte de integração em Go que sobe MariaDB + API reais via testcontainers e exercita a API como um cliente faz.

-----

## 🏗️ Arquitetura

```
crates/
├── api-http/  HTTP: rotas, middlewares, handlers, JWT, formato de fio
├── app/       casos de uso, commands, queries, autorização, transação
├── infra/     banco, repositórios, DQL de leitura, caches, logging
└── domain/    modelos, table modules, geração de ids, hashing
```

```
api-http ──► app ──► infra ──► domain
                       │          ▲
                       └──────────┘
```

Cada camada publica traits e esconde implementações: `ProductRepository` é público, `mariadb::ProductMariadbRepository` é `pub(crate)`. Nada fora da camada nomeia um tipo concreto — o provider devolve `impl Trait`, e é isso que permite trocar o repositório SQL por um dublê em memória sem que nenhum chamador perceba.

**Caminho de uma requisição:** `main` → `RequestId` → `Logging` → `Recover` → `Timeout` → `CORS` → `Token` → handler → UseCase → Repository/DQL → resposta negociada.

Não há *composition root*: o `api-http` tem o próprio `main`, lê os segredos do ambiente e chama `app::register`, que encadeia `infra` e `domain`. O `main` não declara `infra` nem `domain` como dependência.

-----

## 🌐 Endpoints

| Domínio | Rotas |
|---|---|
| **Servidor** | `GET /info` |
| **Bootstrap** | `POST /setup` — única porta de entrada num sistema sem usuários; responde 409 depois do primeiro |
| **Autenticação** | `POST /auth/login` · `POST /auth/refresh` · `POST /auth/logout` |
| **Conta** | `GET /account` · `PUT /account` · `PUT /account/password` |
| **Produtos** | `GET` `POST` `/products` · `GET` `PUT` `DELETE` `/products/{id}` |
| **Contêineres** | `GET` `POST` `/containers` · `GET /containers/summary` · `GET` `PUT` `DELETE` `/containers/{id}` · `POST /containers/{id}/seal` · `POST /containers/{id}/dispatch` |
| **Manifestos** | `POST /manifests/load-item` · `POST /manifests/unload-item` |
| **Usuários (admin)** | `GET` `POST` `/users` · `GET` `PUT` `DELETE` `/users/{id}` · `PUT /users/{id}/password` · `PUT /users/{id}/roles` |
| **Papéis (admin)** | `GET` `POST` `/roles` · `PUT /roles/{id}/permissions` |
| **Metadados do sistema** | `GET /metadata/permissions` — catálogo preenchido em código no boot; sem paginação, filtrável por `?search=` |
| **Métricas** | `GET /metrics` |

Trinta e cinco rotas, sem prefixo de versão. A sessão trafega em cookies `HttpOnly`: um JWT HS256 de curta duração, que carrega o principal inteiro empacotado em FlatBuffers dentro de uma claim, e um *refresh token* opaco (NanoID) revogável por *marker* e rotacionado a cada uso.

-----

## 🛠️ Instalação

### Requisitos

| | |
|---|---|
| **Docker + Compose** | caminho recomendado; é tudo que a stack de desenvolvimento precisa |
| **Rust estável** | para compilar fora de container; `rust-toolchain.toml` fixa canal e componentes |
| **Go 1.25+** | apenas para a suíte de integração |
| **flatc 25.12+** | apenas para regerar os bindings Go da suíte — a API não usa |

### 1. Clone com submódulos

Os schemas FlatBuffers vivem no submódulo `swagger/` — sem ele o `build.rs` não tem o que gerar, e a compilação para antes de começar.

```bash
git clone --recurse-submodules git@github.com:allan-rcos/tachyon-portmaster-back-php.git portmaster
cd portmaster
```

Já clonou sem eles? `git submodule update --init --recursive`.

### 2. Suba a stack

```bash
docker compose up -d
```

O Compose orquestra a ordem inteira: `db` (MariaDB 11) → `migrate` (golang-migrate) → `seed` (`db/seeds/dev.sql`, idempotente) → `app` (a API na porta `8000`).

> A primeira subida inicializa um volume novo do MariaDB e pode levar alguns minutos; o healthcheck tem `start_period` de 180s justamente para não estourar o orçamento de tentativas nesse intervalo.

### 3. Compilação local (opcional)

```bash
cargo build --workspace
cargo run --bin portmaster-api-http
```

A configuração é **inteiramente por variáveis de ambiente** — a mesma imagem serve a stack de desenvolvimento e o pool de testes, cada um apontado para seu banco. Só entra aqui **segredo ou identidade de deploy**: tamanho de pool, capacidade e TTL de cache e estratégia de id são *features* de compilação, não variáveis, porque um `if` em produção sobre uma decisão de arquitetura é um bug esperando o dia de errar.

| Variável | Papel |
|---|---|
| `APP_HOST`, `APP_PORT` | endereço de escuta |
| `APP_ENV` | nome do ambiente, publicado em `GET /info` |
| `APP_REQUEST_TIMEOUT` | teto de tempo de uma requisição, em segundos (30 por padrão) |
| `APP_DB_HOST`, `APP_DB_PORT`, `APP_DB_NAME`, `APP_DB_USER`, `APP_DB_PASSWORD` | banco |
| `APP_DB_SSL_MODE` | `disabled` (padrão), `required` ou `verify_ca` |
| `APP_DB_SSL_CA`, `APP_DB_SSL_VERIFY_CN` | bundle da CA e checagem do nome — só lidos em `verify_ca` |
| `APP_JWT_SECRET` | chave de assinatura HS256 — **mínimo de 32 bytes**, o boot recusa menos |
| `APP_JWT_ISSUER` | emissor, gravado e conferido na claim `iss` |
| `APP_CLUSTER_ID`, `APP_SERVER_ID` | identidade deste processo na composição do Snowflake |
| `APP_CORS_ORIGINS` | origens aceitas, separadas por vírgula; vazio não acrescenta cabeçalho nenhum |

> **Em produção**, troque `APP_JWT_SECRET` por um valor aleatório forte. Não há mais nada a ligar: os cookies de sessão saem `Secure`, `HttpOnly` e `SameSite=Strict` por construção.

> **A sessão não é configurável, e é de propósito.** Validade do token e do refresh, nomes dos cookies, `Secure` e `SameSite` eram seis variáveis (`APP_JWT_TTL`, `APP_REFRESH_TTL`, `APP_JWT_COOKIE_NAME`, `APP_REFRESH_COOKIE_NAME`, `APP_JWT_COOKIE_SECURE`, `APP_JWT_COOKIE_SAME_SITE`) e viraram a `SessionPolicy`, fixada em compilação. Nenhuma era segredo nem identidade de deploy: eram o que a API promete, e duas instâncias da mesma versão não deveriam poder discordar disso — muito menos emitir um cookie cuja validade não bate com a do token que ele carrega. O `Secure` acompanha o perfil de compilação; um build que precise servir HTTP puro passa `--build-arg RUST_DEBUG_ASSERTIONS=on`, que é o que o `docker-compose.yml` e a suíte de integração fazem.

> **Sobre `APP_DB_SSL_MODE`.** O padrão `disabled` é a resposta certa para um banco em `127.0.0.1` ou numa subnet privada — e a errada para qualquer banco gerenciado, que recusa conexão em claro. `required` criptografa sem validar o certificado: resolve escuta passiva, não ataque ativo. `verify_ca` exige `APP_DB_SSL_CA` e valida a cadeia, recusando um certificado que não fecha com a CA configurada em vez de cair para texto claro.

-----

## 🚀 Uso

### 1. Faça o bootstrap do sistema

Não existe usuário semeado — o primeiro administrador nasce de uma chamada explícita, que se recusa com `409` a partir da segunda. O papel criado aí recebe **todas** as permissões registradas pelos casos de uso, e quem acabou de digitar a senha já sai logado: os cookies de sessão vêm no `201`.

```bash
curl -c jar.txt -X POST localhost:8000/setup -H 'Content-Type: application/json' \
     -d '{"name":"Admin","email":"admin@portmaster.local","password":"Portmaster1"}'
```

### 2. Autentique-se

```bash
curl -c jar.txt -X POST localhost:8000/auth/login \
     -H 'Content-Type: application/json' \
     -d '{"email":"admin@portmaster.local","password":"Portmaster1"}'
```

### 3. Consuma a API

```bash
# JSON, por negociação de conteúdo
curl -b jar.txt localhost:8000/products -H 'Accept: application/json'

# FlatBuffers binário — o formato nativo
curl -b jar.txt localhost:8000/products -H 'Accept: application/x-flatbuffers' --output products.fb

curl localhost:8000/info
docker compose logs -f app
```

> Sem `Content-Type`, o corpo é lido como FlatBuffers; sem `Accept`, a resposta sai em JSON. A assimetria é deliberada: quem manda corpo sem anunciar o tipo é um cliente nosso falando o formato nativo, e quem não pede formato nenhum costuma ser um humano com um `curl`.

Ciclo de vida de um contêiner: crie-o, embarque itens com `POST /manifests/load-item` (o peso corrente é mantido na mesma transação da escrita do item), sele com `POST /containers/{id}/seal` e despache com `POST /containers/{id}/dispatch`.

-----

## ✅ Qualidade

| Comando | O que faz |
|---|---|
| `cargo fmt --all --check` | formatação |
| `cargo clippy --workspace --all-targets -- -D warnings` | lint, incluindo o código de teste |
| `cargo test --workspace` | testes unitários — regras de domínio, espinha transacional, wire |
| `dagger call integration-test` | suíte de integração em Go (precisa de Docker) |
| `dagger call generate-fbs-go` | regera os bindings Go da suíte de testes |
| `cargo doc --workspace --no-deps --open` | a documentação de API, gerada do próprio código |

**A linha divisória entre as suítes:** se um comportamento é observável por uma requisição e uma resposta, é integração; se é uma regra ou um desvio, é unitário. Os testes unitários batem direto nos *table modules* — é onde as regras existem — e nos casos de uso, verificando *commit* no caminho feliz, *rollback* em qualquer falha e o guarda de `403`. A suíte de integração é escrita como **histórias** (sessão, administração, pátio) que sobem MariaDB em tmpfs e um pool de APIs reais via testcontainers-go.

O CI ([GitHub Actions](.github/workflows/ci.yml)) roda dois jobs independentes: o de Rust (`fmt`, `clippy`, testes, build de release) e a suíte Go — esta última falhando se os bindings FlatBuffers comitados estiverem defasados.

-----

## 📚 Documentação

| | |
|---|---|
| [`docs/adr/`](docs/adr/) | **por que** as coisas são como são |
| [`db/README.md`](db/README.md) | schema, migrações, seeds, tipos de coluna e soft-delete |
| [`.github/README.md`](.github/README.md) | os workflows e o que cada job cobre |
| [`tests/integration/README.md`](tests/integration/README.md) | as histórias e o harness de testcontainers |
| `cargo doc` | cada módulo documenta a decisão que o produziu, não o que o código já diz |

-----

## ✏️ Contribuir

Contribuições são bem-vindas. Antes de abrir um PR:

1. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` e `cargo test --workspace` precisam passar; para mudanças na borda da API, `dagger call integration-test` também.
2. Mantenha a direção das dependências e a convenção de contrato/implementação: trait público, tipo concreto `pub(crate)`, provider devolvendo `impl Trait`.
3. Regra nova vive no *table module*, não no handler nem no repositório.
4. Mudou schema `.fbs`? Regere os bindings Go e comite o resultado — os tipos Rust saem do build.
5. Decisão estrutural merece um ADR em [`docs/adr/`](docs/adr/).

-----

## 🔓 Licença

[![MIT](https://img.shields.io/badge/MIT-green?style=for-the-badge)](#)
