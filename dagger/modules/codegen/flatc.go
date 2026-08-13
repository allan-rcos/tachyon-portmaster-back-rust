package main

import (
	"dagger/codegen/internal/dagger"
	"fmt"
)

// withFlatc instala o flatc num container Debian.
func withFlatc(ctr *dagger.Container) *dagger.Container {
	url := fmt.Sprintf(
		"https://github.com/google/flatbuffers/releases/download/v%s/Linux.flatc.binary.g++-13.zip",
		flatcVersion,
	)
	return ctr.
		WithExec([]string{"sh", "-c",
			"apt-get update -qq && apt-get install -y -qq --no-install-recommends curl unzip ca-certificates >/dev/null"}).
		WithExec([]string{"curl", "-fsSL", "-o", "/tmp/flatc.zip", url}).
		WithExec([]string{"unzip", "-q", "-o", "/tmp/flatc.zip", "-d", "/usr/local/bin"}).
		WithExec([]string{"chmod", "+x", "/usr/local/bin/flatc"})
}
