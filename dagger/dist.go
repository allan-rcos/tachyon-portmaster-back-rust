package main

import "dagger/back-rust/internal/dagger"

// Dist monta os dois tarballs de produção e devolve o diretório dist/.
//
//	dagger call dist --version 1.0.0 export --path dist
func (m *BackRust) Dist(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
	// +optional
	version string,
) *dagger.Directory {
	return dag.Artifact().Build(dagger.ArtifactBuildOpts{
		Source: source, Version: version,
	})
}
