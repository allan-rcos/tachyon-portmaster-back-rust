package main

import "dagger/codegen/internal/dagger"

// GenerateFbsGo regenera os bindings Go da suíte de integração.
//
// Os dois `perl -i` parecem cosméticos e não são: o flatc não reescreve as
// referências cruzadas quando `--go-namespace` colapsa os namespaces, e deixa
// pendurados os imports `API__Fbs__X "API/Fbs/X"` e os prefixos correspondentes.
// Sem os dois o pacote não compila, e a mensagem fala de import não usado —
// nunca de namespace.
func (m *Codegen) GenerateFbsGo(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) *dagger.Directory {
	return withFlatc(dag.Container().From("golang:1.25")).
		WithMountedDirectory("/work", source).
		WithWorkdir("/work").
		WithExec([]string{"sh", "-c", `
			set -e
			out=tests/integration/internal/fbs
			rm -rf "$out" && mkdir -p "$out"

			flatc --go --go-namespace fbs \
			      -o tests/integration/internal \
			      swagger/flatbuffers/schemas/*.fbs

			perl -i -ne 'print unless /^\s*API__Fbs__\w+ "API\/Fbs\/\w+"$/' "$out"/*.go
			perl -i -pe 's/API__Fbs__\w+\.//g' "$out"/*.go
			gofmt -w "$out"
		`}).
		Directory("/work/tests/integration/internal/fbs")
}
