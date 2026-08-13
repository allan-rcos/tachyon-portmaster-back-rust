package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// Clippy roda o linter com aviso tratado como erro.
//
// `-D warnings` é o que faz o lint valer: sem ele o clippy sai com zero e a
// única consequência de um aviso é ninguém ler.
func (m *BackRust) Clippy(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithExec([]string{"cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"}).
		Stdout(ctx)
}
