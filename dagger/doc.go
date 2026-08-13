package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// O redirect que o GitHub Pages precisa.
//
// O `target/doc` do cargo não tem index.html próprio — o `cargo doc --open`
// adivinha um crate. Uma página publicada precisa de um ponto de entrada de
// verdade, e é este.
const rustdocIndex = `<!doctype html>
<meta charset="utf-8">
<title>Portmaster — referência da API</title>
<meta http-equiv="refresh" content="0; url=portmaster_domain/index.html">
<p>Redirecionando para <a href="portmaster_domain/index.html">portmaster_domain</a>.
`

// Doc renderiza a referência de API e devolve o diretório docs/rustdoc.
//
//	dagger call doc export --path docs/rustdoc
//
// `RUSTDOCFLAGS=-D warnings` é o que faz isto valer como verificação: sem ele um
// link quebrado numa doc comment sai como aviso e ninguém lê. Com ele, o job
// falha — que é o motivo de esta função entrar no `ci`.
//
// `--document-private-items` porque o que interessa aqui é a implementação: os
// itens públicos são poucos, e a documentação existe para quem mexe no código.
//
// R6.1 — a função devolve o diretório e não o escreve. O script equivalente
// fazia `rm -rf docs/rustdoc` antes de copiar, e o delete-first era deliberado:
// sem ele um módulo renomeado deixava a página antiga para trás para sempre,
// alcançável por quem tivesse o link e descrevendo código que não existe mais.
// Com `export`, o Dagger substitui o diretório inteiro e a propriedade se
// mantém sem ninguém precisar lembrar.
func (m *BackRust) Doc(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) *dagger.Directory {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithEnvVariable("RUSTDOCFLAGS", "-D warnings").
		WithExec([]string{
			"cargo", "doc", "--workspace", "--no-deps", "--document-private-items",
		}).
		// O target/ é um cache montado, então o conteúdo é copiado para fora
		// dele antes de virar Directory: devolver um caminho dentro do mount
		// traria o cache inteiro junto.
		WithExec([]string{"sh", "-c", "mkdir -p /out && cp -r target/doc/. /out/"}).
		WithNewFile("/out/index.html", rustdocIndex).
		Directory("/out")
}

// CheckDoc falha se a documentação não construir — o mesmo que Doc, sem a
// saída. É o que o `ci` chama, porque ali interessa o veredito e não o HTML.
func (m *BackRust) CheckDoc(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithEnvVariable("RUSTDOCFLAGS", "-D warnings").
		WithExec([]string{
			"cargo", "doc", "--workspace", "--no-deps", "--document-private-items",
		}).
		Stdout(ctx)
}
