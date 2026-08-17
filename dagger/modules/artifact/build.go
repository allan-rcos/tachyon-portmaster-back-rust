package main

import (
	"context"
	"dagger/artifact/internal/dagger"
	"fmt"
	"strings"
)

// Build monta os dois tarballs e devolve o diretório dist/.
//
// O --locked não é zelo: o artefato sai das versões exatas que foram testadas.
// Uma release que pegou em silêncio um patch novo de uma dependência é uma
// release que ninguém verificou.
//
// O binário é `strip`ado porque os símbolos de debug são a maior parte dele e
// nada no servidor os lê. A cópia com símbolos fica em target/ para quem
// precisar resolver um trace, e o .sha256 diz a que build ela pertence.
func (m *Artifact) Build(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["target", "dist", "docs", ".git", ".github", "**/.git", "tmp"]
	source *dagger.Directory,
	// Vazio lê de [workspace.package] no Cargo.toml.
	// +optional
	version string,
) (*dagger.Directory, error) {
	if version == "" {
		v, err := m.Version(ctx, source.File("Cargo.toml"))
		if err != nil {
			return nil, err
		}
		version = v
	}
	version = strings.TrimPrefix(version, "v")
	if version == "" {
		return nil, fmt.Errorf("não consegui determinar a versão")
	}

	api := fmt.Sprintf("portmaster-api-%s-linux-x86_64.tar.zst", version)
	migrations := fmt.Sprintf("portmaster-migrations-%s.tar.zst", version)
	image := fmt.Sprintf("portmaster-api-%s-image.tar", version)

	// A imagem sai do MESMO Dockerfile que a suíte de integração sobe, e é essa
	// a razão de ela vir daqui em vez de ser montada à mão em volta do binário
	// acima: o que se publica é o que foi exercitado.
	//
	// Ela traz um binário diferente do que vai no tarball, e isso é esperado —
	// o tarball é compilado para musl a partir da imagem Debian do toolchain, e
	// o da imagem nasce na `rust:1-alpine`, que é musl nativa. Mesmo código, dois
	// caminhos de compilação, dois artefatos para dois modos de implantar.
	//
	// `off` nas debug assertions: é a imagem de produção. Quem quer a que serve
	// HTTP puro pede por ela em `dagger call api-image`.
	imageTar := source.DockerBuild(dagger.DirectoryDockerBuildOpts{
		BuildArgs: []dagger.BuildArg{
			{Name: "RUST_DEBUG_ASSERTIONS", Value: "off"},
		},
	}).AsTarball()

	return dag.Toolchain().Dev(dagger.ToolchainDevOpts{Source: source}).
		WithExec([]string{
			"cargo", "build", "--release", "--locked",
			"--target", muslTarget, "--bin", apiBinary,
		}).
		WithExec([]string{"mkdir", "-p", "/stage/api", "/stage/migrations", "/out"}).
		WithExec([]string{"install", "-m", "0755",
			"target/" + muslTarget + "/release/" + apiBinary, "/stage/api/"}).
		WithExec([]string{"strip", "/stage/api/" + apiBinary}).
		WithExec([]string{"cp", "-r", "db/migrations", "db/seeds", "/stage/migrations/"}).
		// `tar -C <dir> -c .` põe o conteúdo na RAIZ do arquivo, sem diretório
		// versionado no topo. É diferente do artefato em PHP, que traz
		// `portmaster-api-<versão>-linux-x86_64/` como raiz, e a role do Ansible
		// que desempacota precisa saber de qual das duas está tratando.
		WithExec([]string{"sh", "-c", fmt.Sprintf(
			`tar -C /stage/api -c . | zstd -19 -q -o /out/%s
			 tar -C /stage/migrations -c . | zstd -19 -q -o /out/%s`, api, migrations)}).
		// A imagem não leva zstd por fora: as camadas dela já saem comprimidas
		// dentro do próprio arquivo OCI, e uma segunda passada custaria minutos
		// de CPU para não tirar quase nada.
		WithFile("/out/"+image, imageTar).
		// Caminhos relativos dentro do arquivo de checksum, para o `sha256sum -c`
		// funcionar a partir de dist/ independentemente de onde foi construído.
		WithExec([]string{"sh", "-c",
			`cd /out && for f in *.tar.zst *.tar; do sha256sum "$f" > "$f.sha256"; done`}).
		Directory("/out"), nil
}
