// BackRust — as verificações e o empacotamento da API em Rust.
//
// ---------------------------------------------------------------------------
// COMO ESTE DIRETÓRIO É ORGANIZADO
//
// Um arquivo por função, com o nome do comando:
//
//	main.go          só o tipo e esta explicação
//	ci.go            dagger call ci
//	fmt.go           dagger call fmt
//	clippy.go        dagger call clippy
//	lintexports.go   dagger call lint-exports
//	doc.go           dagger call doc
//	test.go          dagger call test
//	dist.go          dagger call dist
//	version.go       dagger call version
//	fbsgo.go         dagger call generate-fbs-go
//	fbscheck.go      dagger call check-fbs-go
//	integration.go   dagger call integration-test
//
// O QUE É GERADO E O QUE VOCÊ ESCREVE
//
//	escrito à mão   dagger.json, go.mod, go.sum, *.go, modules/
//	gerado          dagger.gen.go, internal/dagger/, internal/telemetry/
//
// ---------------------------------------------------------------------------
// Esta API é uma segunda implementação da MESMA API que back-php serve, e o
// contrato de artefato é deliberadamente idêntico: os mesmos nomes de asset, o
// mesmo par de tarballs, o mesmo .sha256 ao lado. A infraestrutura escolhe qual
// das duas implanta sem saber que a linguagem mudou.
//
// O que difere é o conteúdo: aqui o tarball da API é um binário estático musl,
// e o do PHP é src/ mais vendor/ minificados. É a role do Ansible que precisa
// saber a diferença — ver ansible/roles/rust-api na infraestrutura.
// ---------------------------------------------------------------------------
package main

type BackRust struct{}
