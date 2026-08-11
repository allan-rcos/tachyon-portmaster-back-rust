# Portmaster API — build estático em musl, imagem final sem toolchain.
#
# Duas etapas: a primeira compila, a segunda só carrega o binário. O que sai daqui
# são cerca de vinte megabytes contra os dois gigabytes da imagem de build, e
# nada de compilador, gerenciador de pacote ou código-fonte fica no que roda em
# produção — a superfície de ataque de um container é o que existe dentro dele.
#
# ## Por que musl e não glibc
#
# O binário fica **estático**: não procura nenhuma biblioteca do sistema em
# tempo de execução, então a imagem final não precisa ter uma. É o que permite a
# segunda etapa ser uma alpine quase vazia, e é o que faz o mesmo binário rodar
# igual numa distribuição diferente.
#
# ## O que a compilação precisa que o código não mostra
#
# `build.rs` gera os tipos de wire a partir de `swagger/flatbuffers/schemas/*.fbs`
# **durante o build**, em Rust puro — não há `flatc` aqui, de propósito: ele não
# existiria na imagem final e o build ficaria refém de uma ferramenta externa.
# Os schemas são um submódulo; um clone sem `--recursive` falha nesta etapa, com
# a mensagem que o próprio `build.rs` escreve.

# --- Etapa 1: compilar ------------------------------------------------------

FROM rust:1-alpine AS builder

# `musl-dev` traz o `cc` que as dependências com código C precisam — o `ring`, do
# TLS, é a que importa. `perl` não entra: nada aqui usa OpenSSL, e o TLS do banco
# é rustls.
RUN apk add --no-cache musl-dev

WORKDIR /build

# Só o que participa da compilação. O `.dockerignore` já barra o resto; listar
# aqui o que entra deixa explícito que uma mudança em `docs/` não invalida o
# cache desta camada.
COPY Cargo.toml Cargo.lock rust-toolchain.toml .clippy.toml ./
COPY crates ./crates
COPY swagger ./swagger

# O `xtask` não vai para a imagem, mas o seu `Cargo.toml` precisa existir: ele é
# membro do workspace, e o cargo carrega o manifesto de todos os membros antes de
# decidir o que compilar. O `default-members` mantém o crate fora do build.
COPY xtask/Cargo.toml ./xtask/Cargo.toml
RUN mkdir -p xtask/src && echo 'fn main() {}' > xtask/src/main.rs

# A `SessionPolicy` liga o `Secure` dos cookies de sessão pelo perfil de
# compilação, e esta imagem é release — então o cookie sai `Secure` e um cliente
# que fale HTTP puro nunca o devolve. É o correto para produção, e é o que
# inviabilizaria o compose de desenvolvimento e a suíte de integração, que sobem
# esta mesma imagem em `http://`.
#
# Daí o argumento: ele liga as debug assertions dentro do build de release, que é
# o que a `SessionPolicy` lê. Fica `off` por padrão de propósito — quem quer a
# imagem que serve HTTP puro tem que pedir por ela.
ARG RUST_DEBUG_ASSERTIONS=off

# Os caches de registro e de artefato ficam em mounts do BuildKit em vez de numa
# camada: uma recompilação reaproveita o que já foi construído sem que nada disso
# vá parar na imagem. Por isso o binário é copiado para fora do `target` ainda
# dentro do mesmo `RUN` — o mount some quando ele termina.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    if [ "$RUST_DEBUG_ASSERTIONS" = "on" ]; then export RUSTFLAGS="-C debug-assertions=on"; fi \
    && cargo build --release --locked --bin portmaster-api-http \
    && cp target/release/portmaster-api-http /usr/local/bin/portmaster-api-http

# --- Etapa 2: rodar ---------------------------------------------------------

FROM alpine:3.22

# `ca-certificates` para o TLS do banco quando `APP_DB_SSL_MODE` estiver ligado.
# Sem isso, `verify_ca` não teria cadeia contra o que validar e o boot falharia —
# com razão, mas por um motivo que não é o que o operador configurou.
RUN apk add --no-cache ca-certificates \
    && adduser -D -H -u 10001 portmaster

COPY --from=builder /usr/local/bin/portmaster-api-http /usr/local/bin/portmaster-api-http

# Não-root. O processo não escreve em disco, não escuta em porta privilegiada e
# não precisa de nada que exija privilégio; rodar como root seria só o padrão.
USER portmaster

# O mesmo par que o `config.rs` lê. Declarados aqui para que `docker inspect`
# responda em que porta a imagem escuta sem ninguém abrir o código.
ENV APP_HOST=0.0.0.0 \
    APP_PORT=8000
EXPOSE 8000

# `/info` é a única rota pública com corpo, e responder nela prova mais do que um
# TCP aberto: o processo passou pelo `register`, o que significa que o banco
# respondeu e o catálogo de permissões foi preenchido.
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
    CMD wget -qO- "http://127.0.0.1:${APP_PORT}/info" >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/portmaster-api-http"]
