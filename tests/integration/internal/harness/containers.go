package harness

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"time"

	"github.com/docker/go-connections/nat"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/network"
	"github.com/testcontainers/testcontainers-go/wait"
)

const (
	apiImageTag = "portmaster-api:itest"
	dbAlias     = "mariadb"
	dbRootPass  = "root"
)

// buildAPIImage builds the application image from the repo Dockerfile exactly
// once; every API container in the pool then starts from this tag.
//
// RUST_DEBUG_ASSERTIONS is what keeps the session cookies usable here. The
// SessionPolicy marks them Secure in a release build, and Go's cookie jar
// refuses to send a Secure cookie over the http:// URL these containers expose —
// so without the arg the whole session story would fail on a cookie the server
// did set.
//
// INTEGRATION_API_PREBUILT skips the build entirely, and the Dagger pipeline is
// what sets it: there the image is built by Dagger — same Dockerfile, same build
// arg — and loaded into the daemon under this tag before the suite starts.
// Building again would rebuild what is already there, and the cost is not the
// compile (that one is cached) but the build context: this repository lives on a
// FUSE filesystem, where transferring it and re-checking the COPY layers costs
// around seventy seconds per run.
//
// The variable is opt-in rather than a presence check on the tag. Reusing
// whatever image happens to carry the tag would silently test a stale binary the
// first time someone forgot to rebuild — so an unset variable always builds,
// which is what keeps `go test` run by hand correct on its own.
func buildAPIImage(ctx context.Context, repoRoot string) error {
	if os.Getenv("INTEGRATION_API_PREBUILT") != "" {
		return nil
	}

	cmd := exec.CommandContext(ctx, "docker", "build",
		"--build-arg", "RUST_DEBUG_ASSERTIONS=on",
		"-t", apiImageTag, repoRoot)
	cmd.Stdout = os.Stderr
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("docker build %s: %w", apiImageTag, err)
	}
	return nil
}

// startMariaDB launches one shared MariaDB whose data dir lives on tmpfs (RAM),
// reachable inside the network as "mariadb" and, for the test process itself,
// through its mapped host port.
func startMariaDB(ctx context.Context, networkName string) (testcontainers.Container, string, error) {
	req := testcontainers.ContainerRequest{
		Image: "mariadb:11",
		// Matches the dev compose stack: the marker purge is a MariaDB EVENT,
		// and the scheduler that fires events is off by default. Migrations
		// create the event either way, so without this the schema would apply
		// but the purge would silently never run.
		// --default-time-zone matches the dev compose stack. Nothing depends
		// on it: time is stored as BIGINT epoch milliseconds, written by the
		// application, so no column is interpreted against a session zone. It
		// is here so that a NOW() typed into a shell against this container
		// answers the same way it would against the dev stack.
		Cmd:          []string{"--event-scheduler=ON", "--default-time-zone=+00:00"},
		Env:          map[string]string{"MARIADB_ROOT_PASSWORD": dbRootPass},
		ExposedPorts: []string{"3306/tcp"},
		Tmpfs:        map[string]string{"/var/lib/mysql": "rw"},
		Networks:     []string{networkName},
		NetworkAliases: map[string][]string{
			networkName: {dbAlias},
		},
		// Waiting on the listening port is not enough: the MariaDB image boots a
		// *temporary* server to initialise the data directory, then shuts it
		// down and starts the real one. The port is open for that first server
		// too, so a port-based wait hands back a database that is about to
		// disappear — connections opened against it die with "unexpected EOF"
		// moments later. Only a query that actually answers proves the final
		// server is up.
		WaitingFor: wait.ForSQL("3306/tcp", "mysql", func(host string, port nat.Port) string {
			return fmt.Sprintf("root:%s@tcp(%s:%s)/", dbRootPass, host, port.Port())
		}).
			WithQuery("SELECT 1").
			WithStartupTimeout(120 * time.Second),
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		return nil, "", fmt.Errorf("start mariadb: %w", err)
	}

	mapped, err := container.MappedPort(ctx, "3306/tcp")
	if err != nil {
		return nil, "", fmt.Errorf("mariadb mapped port: %w", err)
	}

	return container, mapped.Port(), nil
}

// startAPI launches one API container bound to the given database on the shared
// MariaDB, and returns it together with its externally reachable base URL.
func startAPI(ctx context.Context, networkName, dbName string) (testcontainers.Container, string, error) {
	req := testcontainers.ContainerRequest{
		Image:        apiImageTag,
		ExposedPorts: []string{"8000/tcp"},
		Networks:     []string{networkName},
		Env: map[string]string{
			"APP_HOST":        "0.0.0.0",
			"APP_PORT":        "8000",
			"APP_WORKER_NUM":  "2",
			"APP_DB_HOST":     dbAlias,
			"APP_DB_PORT":     "3306",
			"APP_DB_NAME":     dbName,
			"APP_DB_USER":     "root",
			"APP_DB_PASSWORD": dbRootPass,
			// HS256 rejects a secret shorter than its 32-byte digest.
			"APP_JWT_SECRET": "integration-test-secret-32-bytes-min",
		},
		WaitingFor: wait.ForHTTP("/info").
			WithPort("8000/tcp").
			WithStartupTimeout(90 * time.Second),
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		return nil, "", fmt.Errorf("start api (%s): %w", dbName, err)
	}

	host, err := container.Host(ctx)
	if err != nil {
		return nil, "", err
	}
	mapped, err := container.MappedPort(ctx, "8000/tcp")
	if err != nil {
		return nil, "", err
	}

	return container, fmt.Sprintf("http://%s:%s", host, mapped.Port()), nil
}

// restartAPI stops and starts an API container, waits for it to serve /info
// again, and returns its new base URL.
//
// This is how the application's boot-time state is rebuilt after a reset: the
// ENGINE=MEMORY registries (permissions, marker groups) are filled at
// WorkerStart and nowhere else, so dropping the schema without restarting would
// leave the server pointing at a catalogue that no longer exists.
//
// It returns the URL rather than mutating nothing because Docker assigns a
// *fresh* host port on every start: the mapping the container had before the
// stop is gone, and any URL captured earlier now refuses connections.
func restartAPI(ctx context.Context, container testcontainers.Container) (string, error) {
	timeout := 30 * time.Second
	if err := container.Stop(ctx, &timeout); err != nil {
		return "", fmt.Errorf("stop api: %w", err)
	}
	if err := container.Start(ctx); err != nil {
		return "", fmt.Errorf("start api: %w", err)
	}

	// Start() returns once the container is running, which is well before
	// OpenSwoole has booted its workers and re-registered the metadata.
	strategy := wait.ForHTTP("/info").
		WithPort("8000/tcp").
		WithStartupTimeout(90 * time.Second)

	if err := strategy.WaitUntilReady(ctx, container); err != nil {
		return "", fmt.Errorf("api not ready after restart: %w", err)
	}

	host, err := container.Host(ctx)
	if err != nil {
		return "", err
	}
	mapped, err := container.MappedPort(ctx, "8000/tcp")
	if err != nil {
		return "", err
	}

	return fmt.Sprintf("http://%s:%s", host, mapped.Port()), nil
}

// newNetwork creates the bridge network the MariaDB and API containers share.
func newNetwork(ctx context.Context) (*testcontainers.DockerNetwork, error) {
	return network.New(ctx)
}
