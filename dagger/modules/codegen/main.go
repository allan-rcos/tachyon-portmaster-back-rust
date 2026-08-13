// Codegen — os bindings Go que a suíte de integração consome.
//
//	main.go      o tipo e a versão do flatc
//	flatc.go     instalar o compilador
//	fbsgo.go     gerar
//	fbscheck.go  conferir se o commitado está em dia
//
// Os schemas são os MESMOS do repositório em PHP — o submódulo `swagger` é o
// mesmo em ambos. Um .fbs novo obriga a regerar bindings nos dois, e a
// divergência aparece como incompatibilidade de wire em runtime, não como erro
// de build.
//
// R6.1 — nenhuma função escreve no repositório. Elas devolvem Directory, e quem
// grava é o `export` de quem chamou.
package main

// A mesma versão usada no repositório em PHP e no front. O binário oficial da
// release evita compilar o FlatBuffers inteiro só para gerar bindings.
const flatcVersion = "25.12.19"

type Codegen struct{}
