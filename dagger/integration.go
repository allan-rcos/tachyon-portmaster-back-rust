package main

import (
	"context"
	"dagger/back-rust/internal/dagger"
	"strings"
)

// IntegrationTest roda a suíte Go de ponta a ponta contra esta API.
//
//	dagger call integration-test
//	dagger call integration-test --args -run,TestYardStory
//
// ---------------------------------------------------------------------------
// A suíte é a MESMA que o repositório em PHP roda: os mesmos schemas, os mesmos
// bindings, as mesmas histórias. É isso que a torna a prova de que as duas
// implementações servem o mesmo contrato — se um teste passa lá e falha aqui, a
// divergência é de comportamento, não de linguagem.
//
// O daemon e os testes rodam no MESMO container. A tentativa natural — dockerd
// como serviço Dagger à parte, alcançado por DOCKER_HOST — chega perto e falha
// no fim: o testcontainers pede ao daemon uma porta publicada e recebe algo como
// 32768, que é uma porta do HOST DO DAEMON. Do container de teste,
// `127.0.0.1:32768` é outro lugar, e a suíte morre com connection refused depois
// de minutos construindo a imagem.
// ---------------------------------------------------------------------------
func (m *BackRust) IntegrationTest(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
	// Argumentos extras repassados ao `go test`.
	// +optional
	args []string,
	// +optional
	poolSize string,
) (string, error) {
	testCmd := "exec go test ./... -count=1 -timeout 20m"
	for _, a := range args {
		testCmd += " '" + strings.ReplaceAll(a, "'", `'\''`) + "'"
	}

	ctr := dag.Container().
		From("docker:28-dind").
		WithExec([]string{"apk", "add", "--no-cache", "git"}).
		WithDirectory("/usr/local/go",
			dag.Container().From("golang:1.25-alpine").Directory("/usr/local/go")).
		WithEnvVariable("PATH", "/usr/local/go/bin:/go/bin:${PATH}",
			dagger.ContainerWithEnvVariableOpts{Expand: true}).
		WithEnvVariable("GOPATH", "/go").
		WithMountedCache("/var/lib/docker", dag.CacheVolume("rust-dind")).
		WithMountedCache("/go/pkg/mod", dag.CacheVolume("rust-go-mod")).
		WithMountedCache("/root/.cache/go-build", dag.CacheVolume("rust-go-build")).
		// O Ryuk não tem o que vigiar aqui: o daemon inteiro é descartado no fim.
		WithEnvVariable("TESTCONTAINERS_RYUK_DISABLED", "true").
		WithMountedDirectory("/work", source).
		WithWorkdir("/work")

	if poolSize != "" {
		ctr = ctr.WithEnvVariable("INTEGRATION_POOL_SIZE", poolSize)
	}

	return ctr.
		WithExec([]string{"sh", "-c", `
			set -e
			dockerd --host=unix:///var/run/docker.sock \
			        --storage-driver=overlay2 >/tmp/dockerd.log 2>&1 &
			for i in $(seq 1 60); do
				docker info >/dev/null 2>&1 && break
				sleep 1
			done
			docker info >/dev/null 2>&1 || { echo "dockerd nao subiu"; cat /tmp/dockerd.log; exit 1; }
			cd tests/integration
			` + testCmd + `
		`}, dagger.ContainerWithExecOpts{
			InsecureRootCapabilities: true,
		}).
		Stdout(ctx)
}
