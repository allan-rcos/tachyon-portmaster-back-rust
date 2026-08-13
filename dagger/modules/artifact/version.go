package main

import (
	"context"
	"dagger/artifact/internal/dagger"
	"fmt"
	"regexp"
	"strings"
)

// A versão vive em [workspace.package] do Cargo.toml da raiz, que é o mesmo
// campo que o release.yml lê para decidir se publica.
var reWorkspaceVersion = regexp.MustCompile(`(?s)\[workspace\.package\].*?\bversion\s*=\s*"([^"]+)"`)

// Version devolve a versão declarada no Cargo.toml do workspace.
//
// Lida aqui, em Go, e não por `cargo metadata` num container: é um campo de um
// arquivo TOML, e subir a imagem do Rust para lê-lo custaria mais do que o
// passo inteiro. O `cargo metadata` do script fazia exatamente isso, e ainda
// extraía o campo com grep sobre o JSON.
func (m *Artifact) Version(
	ctx context.Context,
	// +defaultPath="/Cargo.toml"
	cargoToml *dagger.File,
) (string, error) {
	contents, err := cargoToml.Contents(ctx)
	if err != nil {
		return "", err
	}
	g := reWorkspaceVersion.FindStringSubmatch(contents)
	if g == nil {
		return "", fmt.Errorf("o Cargo.toml não declara version em [workspace.package]")
	}
	return strings.TrimSpace(g[1]), nil
}
