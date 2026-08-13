package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// CheckFbsGo falha se os bindings Go commitados estiverem defasados.
func (m *BackRust) CheckFbsGo(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Codegen().CheckFbsGo(ctx, dagger.CodegenCheckFbsGoOpts{Source: source})
}
