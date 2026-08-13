package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// Version devolve a versão declarada em [workspace.package] do Cargo.toml.
func (m *BackRust) Version(
	ctx context.Context,
	// +defaultPath="/Cargo.toml"
	cargoToml *dagger.File,
) (string, error) {
	return dag.Artifact().Version(ctx, dagger.ArtifactVersionOpts{CargoToml: cargoToml})
}
