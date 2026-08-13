package main

import "dagger/back-rust/internal/dagger"

// GenerateFbsGo regenera os bindings Go da suíte de integração.
//
//	dagger call generate-fbs-go export --path tests/integration/internal/fbs
func (m *BackRust) GenerateFbsGo(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) *dagger.Directory {
	return dag.Codegen().GenerateFbsGo(dagger.CodegenGenerateFbsGoOpts{Source: source})
}
