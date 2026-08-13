// Artifact — os dois tarballs de release da API em Rust.
//
//	main.go     o tipo e o alvo
//	build.go    dagger call dist
//	version.go  dagger call version
//
// ---------------------------------------------------------------------------
// Este módulo é o antigo scripts/build-dist.sh, REESCRITO.
//
// As migrations NÃO viajam com a API, e isso é desenho: a máquina que roda a
// API é a menos autorizada a alterar o schema, e um deploy que PODE migrar é um
// deploy que migra por acidente. Separar os pacotes torna aplicar uma migration
// um ato deliberado de quem enxerga o banco.
//
// O que continua sendo `withExec` são as ferramentas — cargo, strip, tar, zstd.
// O que virou Go é a decisão: qual alvo, o que entra em cada pacote, como o
// nome é formado.
// ---------------------------------------------------------------------------
//
// O módulo se chama `artifact` e não `dist` por consistência com os outros
// repositórios, onde o .gitignore traz um `dist/` sem barra inicial que faria o
// Dagger carregar o módulo vazio. Aqui o padrão é ancorado (`/dist`) e não
// haveria colisão, mas o nome igual em toda parte vale mais que a exceção.
package main

// O alvo do artefato. O binário é estático para o tarball desempacotar num
// servidor pelado.
const muslTarget = "x86_64-unknown-linux-musl"

// O nome do binário publicado — o mesmo que o Cargo.toml declara como bin.
const apiBinary = "portmaster-api-http"

type Artifact struct{}
