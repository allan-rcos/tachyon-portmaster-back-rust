package main

import "dagger/toolchain/internal/dagger"

// Base devolve a imagem do Rust com o que a ligação estática exige, e nada do
// projeto.
//
// O `musl-gcc` vem do `musl-tools` porque as dependências que carregam C
// precisam de um linker musl para a ligação estática fechar.
//
// Note o que NÃO está aqui: o `rustup target add`. Ver Dev().
func (m *Toolchain) Base() *dagger.Container {
	return dag.Container().
		From("rust:1-slim").
		WithExec([]string{"sh", "-c",
			"apt-get update -qq && apt-get install -y -qq --no-install-recommends " +
				"musl-tools musl-dev zstd binutils pkg-config >/dev/null"})
}

// Dev devolve o Base com o código montado, os caches ligados e o alvo musl
// instalado.
//
// ---------------------------------------------------------------------------
// O `rustup target add` roda DEPOIS de montar o código, e a ordem é o bug.
//
// O repositório traz um rust-toolchain.toml, e o rustup o honra a partir do
// diretório de trabalho: assim que o cargo roda em /work, a toolchain ativa
// passa a ser a que aquele arquivo pede, não a que veio na imagem. Um
// `rustup target add` feito antes instala o std do alvo na toolchain errada, e
// o `cargo build --target` falha com "can't find crate for `core`" — uma
// mensagem que fala de crate e não diz nada sobre toolchain.
//
// Adicionar o alvo já dentro de /work resolve porque aí o rustup e o cargo
// concordam sobre qual toolchain está ativa.
// ---------------------------------------------------------------------------
//
// Os dois caches são separados porque invalidam por motivos diferentes: o
// registry muda quando uma dependência entra, e o target quando o código muda.
// Juntá-los faria uma mudança de código descartar o registry inteiro (R9).
//
// A ordem dos mounts também importa: o diretório entra ANTES do cache de
// target. Invertido, o mount de /work cobriria o de /work/target e o cache não
// valeria nada — sem erro nenhum, só recompilação integral a cada execução.
func (m *Toolchain) Dev(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) *dagger.Container {
	return m.Base().
		// O cache entra DENTRO do CARGO_HOME padrão da imagem, em vez de o
		// CARGO_HOME ser desviado para outro lugar.
		//
		// Parece equivalente e não é: o caminho do registry acaba embutido no
		// binário, em mensagens de panic e metadados que o `strip` não remove.
		// Com CARGO_HOME desviado, o artefato saía 4096 bytes diferente do que
		// um `cargo build` comum produz — mesma versão de rustc, mesmo código,
		// binário diferente. Cachear só o registry mantém os caminhos iguais aos
		// da build de referência.
		WithMountedCache("/usr/local/cargo/registry", dag.CacheVolume("rust-cargo-registry")).
		WithMountedDirectory("/work", source).
		WithMountedCache("/work/target", dag.CacheVolume("rust-target")).
		WithWorkdir("/work").
		WithExec([]string{"rustup", "target", "add", muslTarget}).
		WithExec([]string{"rustup", "component", "add", "rustfmt", "clippy"})
}
