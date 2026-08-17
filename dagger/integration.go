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
//
// A imagem da API NÃO é construída aqui dentro. Ela vem pronta do
// [BackRust.ApiImage], entra como tarball e é carregada no daemon antes do
// `go test` — e o harness, vendo `INTEGRATION_API_PREBUILT`, pula o
// `docker build` que refaria o que já está feito. Ver o doc daquela função para
// por que o build mudou de lado.
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

	image := m.ApiImage(source, "on").AsTarball()

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
		// O harness lê isto e não constrói imagem nenhuma: a que ele usaria já
		// está carregada. Sem a variável ele constrói, que é o que mantém um
		// `go test` rodado à mão, fora daqui, funcionando sozinho.
		WithEnvVariable("INTEGRATION_API_PREBUILT", "1").
		WithMountedFile("/tmp/api-image.tar", image).
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

			# O tarball do Dagger e um arquivo OCI, e o "docker load" devolve
			# "Loaded image:" ou "Loaded image ID:" conforme a referencia venha ou
			# nao anotada. Marcar pelo que sair cobre os dois casos.
			loaded=$(docker load -q -i /tmp/api-image.tar | tail -n1 \
				| sed -e 's/^Loaded image ID: //' -e 's/^Loaded image: //')
			[ -n "$loaded" ] || { echo "docker load nao devolveu imagem"; exit 1; }
			docker tag "$loaded" ` + apiImageTag + `

			cd tests/integration
			` + testCmd + `
		`}, dagger.ContainerWithExecOpts{
			InsecureRootCapabilities: true,
		}).
		Stdout(ctx)
}
