// Toolchain — em que Rust este repositório constrói.
//
//	main.go   o tipo e as versões fixadas
//	base.go   o Rust com o alvo musl
//	dev.go    o base com o código e o cache do cargo
//
// ---------------------------------------------------------------------------
// O alvo é musl, e é estático de propósito.
//
// O binário é ligado estaticamente para o tarball desempacotar num servidor
// pelado — sem runtime, sem biblioteca compartilhada, sem gerenciador de
// pacotes. É a razão de um tarball ainda fazer sentido ao lado da imagem: um
// processo de API que É o servidor web não ganha nada com um runtime de
// container embaixo.
//
// A versão do Rust sai do rust-toolchain.toml do repositório, e não é repetida
// aqui: o `rustup` dentro da imagem oficial lê aquele arquivo e escolhe sozinho,
// então acrescentar um número neste módulo criaria a segunda fonte que a R8
// existe para evitar.
// ---------------------------------------------------------------------------
package main

// O alvo do artefato. Entra no nome do arquivo publicado e é o mesmo que o
// release.yml usa.
const muslTarget = "x86_64-unknown-linux-musl"

type Toolchain struct{}
