package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// Test roda a suíte unitária do workspace.
//
// Só os unitários: tests/integration é uma suíte Go, com testcontainers, e tem
// função própria — ver integration.go.
func (m *BackRust) Test(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithExec([]string{"cargo", "test", "--workspace"}).
		Stdout(ctx)
}
