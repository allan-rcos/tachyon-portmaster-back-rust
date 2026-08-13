package main

import (
	"context"
	"dagger/codegen/internal/dagger"
)

// CheckFbsGo falha se os bindings Go commitados estiverem defasados.
//
// Compara com `diff -r` em vez de `git diff --exit-code`: assim a checagem não
// precisa do .git dentro do container e não suja a árvore de trabalho (R6.1).
func (m *Codegen) CheckFbsGo(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	return dag.Container().
		From("alpine:3.22").
		WithMountedDirectory("/committed", source.Directory("tests/integration/internal/fbs")).
		WithMountedDirectory("/generated", m.GenerateFbsGo(source)).
		WithExec([]string{"sh", "-c",
			`diff -r /committed /generated \
			  || { echo "ERRO: os bindings Go estao defasados — rode: dagger call generate-fbs-go export --path tests/integration/internal/fbs" >&2; exit 1; }
			 echo "bindings Go em dia"`}).
		Stdout(ctx)
}
