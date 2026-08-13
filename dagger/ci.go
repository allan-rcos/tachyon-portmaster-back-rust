package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
)

// Ci roda o que tem de estar verde antes de um merge.
//
// A ordem é a do .github/workflows/ci.yml, e é deliberada: formatação primeiro
// porque é instantânea, clippy depois porque compila, e os testes por último.
// Um erro de formatação não deve custar uma compilação inteira para aparecer.
//
// A suíte de integração fica de fora, como no ci.yml: é um job à parte, com
// custo e teto de tempo próprios.
func (m *BackRust) Ci(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
) (string, error) {
	steps := []struct {
		name string
		run  func(context.Context, *dagger.Directory) (string, error)
	}{
		{"fmt", m.Fmt},
		{"clippy", m.Clippy},
		{"lint-exports", m.LintExports},
		{"doc", m.CheckDoc},
		{"test", m.Test},
	}

	out := ""
	for _, s := range steps {
		r, err := s.run(ctx, source)
		out += "== " + s.name + " ==\n" + r + "\n"
		if err != nil {
			return out, err
		}
	}
	return out, nil
}
