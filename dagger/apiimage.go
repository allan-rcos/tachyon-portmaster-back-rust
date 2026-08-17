package main

import "dagger/back-rust/internal/dagger"

// A tag com que a suíte de integração procura a imagem.
//
// O mesmo literal está em `tests/integration/internal/harness/containers.go`.
// São dois lados do mesmo acordo — quem carrega a imagem e quem a sobe — e não
// há como um importar a constante do outro: um é módulo Dagger, o outro é o
// pacote de teste.
const apiImageTag = "portmaster-api:itest"

// ApiImage constrói a imagem da API a partir do Dockerfile do repositório.
//
//	dagger call api-image --debug-assertions=on export --path api.tar
//
// ---------------------------------------------------------------------------
// É o MESMO Dockerfile que produz a imagem de produção, e isso é o ponto: se a
// suíte de integração subisse uma imagem montada aqui à mão, ela deixaria de
// provar que o Dockerfile constrói e roda — que é metade do valor de ela existir.
//
// O que muda em relação a chamar `docker build` lá dentro do dind é **quem**
// constrói. Aqui o Dagger constrói, e o cache dele é endereçado pelo conteúdo do
// `source`: com o código parado, a chamada inteira é um acerto e nada roda. No
// dind, mesmo com todas as camadas em cache, cada execução ainda pagava a
// transferência do contexto e a conferência das camadas de `COPY` — ~70s numa
// medição, sobre um repositório que mora num sistema de arquivos FUSE.
// ---------------------------------------------------------------------------
func (m *BackRust) ApiImage(
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
	// Liga as debug assertions dentro do build de release. A suíte de integração
	// precisa de `on`; a imagem de produção, não. Ver o `ARG` no Dockerfile.
	// +optional
	// +default="off"
	debugAssertions string,
) *dagger.Container {
	return source.DockerBuild(dagger.DirectoryDockerBuildOpts{
		BuildArgs: []dagger.BuildArg{
			{Name: "RUST_DEBUG_ASSERTIONS", Value: debugAssertions},
		},
	})
}
