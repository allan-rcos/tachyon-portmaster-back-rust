package main

import "dagger/back-rust/internal/dagger"

// Dist monta os artefatos de produção e devolve o diretório dist/.
//
//	dagger call dist --version 1.0.0 export --path dist
//
// São três: o tarball da API, o das migrations e a imagem de contêiner. Os dois
// primeiros são o par que o back-php também publica, com os mesmos nomes; a
// imagem é só desta variante, e existe para quem implanta por contêiner em vez
// de por binário. Cada um leva o seu `.sha256` ao lado.
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
