package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// Fmt confere a formatação sem reescrever nada.
//
// `--check` e não `--write`: numa função Dagger, reescrever seria escrever no
// container e jogar fora. Quem formata é você, na sua máquina.
func (m *BackRust) Fmt(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithExec([]string{"cargo", "fmt", "--all", "--check"}).
		Stdout(ctx)
}
