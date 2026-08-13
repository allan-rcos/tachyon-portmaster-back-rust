package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// LintExports roda o xtask que confere o padrão de export do repositório.
//
// É uma ferramenta do próprio projeto, um crate do workspace — envelopar aqui é
// o certo: a regra que ele aplica é assunto dele, não deste módulo.
func (m *BackRust) LintExports(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithExec([]string{"cargo", "run", "--package", "xtask", "--locked", "--", "lint-exports"}).
		Stdout(ctx)
}
