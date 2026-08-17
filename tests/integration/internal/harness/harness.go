// Package harness stands up the system under test with testcontainers-go: one
// shared tmpfs MariaDB and a pool of API containers, each bound to its own
// database. Tests lease an isolated environment, reset it (migrate drop + up +
// seed), and drive it over HTTP — the harness never touches PHP internals.
package harness

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"testing"

	_ "github.com/go-sql-driver/mysql"
	"github.com/testcontainers/testcontainers-go"
	"golang.org/x/sync/errgroup"
)

// Environment is one isolated {API container + database} slot from the pool.
type Environment struct {
	BaseURL string  // externally reachable API URL, e.g. http://127.0.0.1:49xxx
	DB      *sql.DB // direct connection to this environment's database

	sourceURL  string
	migrateDSN string
	container  testcontainers.Container

	// Ainda não serviu a teste nenhum, e por isso dispensa o Reset.
	fresh bool
}

// Reset returns the environment to a clean, migrated, freshly booted state.
// Each environment resets independently, so parallel tests never contend.
//
// The restart is what clears the API's in-process state, and that is why it is
// not optional here. The view cache holds entries describing rows the drop just
// deleted — its TTL is 30s, longer than a story takes — and the marker map still
// holds the previous test's refresh sessions. Neither lives in the database, so
// dropping the schema does not touch them; only a new process does.
//
// It is *not* about repopulating a catalogue. Permissions and marker groups are
// in-process registries in this port (see ADR 0009), so a schema drop never
// touched them: no table in db/migrations is ENGINE=MEMORY, and the seven that
// exist are all InnoDB.
//
// Seeding then goes through POST /setup rather than SQL, which is the endpoint's
// own reason for existing: a fresh deployment has no user, and no user can be
// created through the guarded routes until one exists.
func (e *Environment) Reset(ctx context.Context, t *testing.T) {
	t.Helper()
	if err := migrateDrop(e.sourceURL, e.migrateDSN); err != nil {
		t.Fatalf("reset drop: %v", err)
	}
	if err := migrateUp(e.sourceURL, e.migrateDSN); err != nil {
		t.Fatalf("reset up: %v", err)
	}
	// The restart moves the API to a new host port, so the environment's URL is
	// re-read here. Tests build their client after Lease() returns, which is
	// what makes handing them a stale URL impossible.
	baseURL, err := restartAPI(ctx, e.container)
	if err != nil {
		t.Fatalf("reset restart: %v", err)
	}
	e.BaseURL = baseURL
}

// Pool hands out environments to parallel tests through a buffered channel.
type Pool struct {
	ch chan *Environment
}

// Lease blocks until an environment is free and returns it after resetting it;
// the environment is returned to the pool automatically when the test ends.
func (p *Pool) Lease(t *testing.T) *Environment {
	t.Helper()
	e := <-p.ch
	t.Cleanup(func() { p.ch <- e })

	// SetupPool já entrega o ambiente migrado e recém-subido, então o primeiro
	// lease não tem o que limpar: derrubar o schema, remigrá-lo e reiniciar a
	// API refaria, em ~12s, exatamente o estado que acabou de ser montado.
	//
	// Com o pool do tamanho do número de histórias, todo lease é um primeiro
	// lease e nenhum Reset roda. A bandeira existe para o caso contrário — uma
	// história a mais que ambientes faz alguém ser alugado duas vezes, e aí o
	// Reset volta a ser obrigatório pelos motivos que o doc dele explica.
	if e.fresh {
		e.fresh = false
		return e
	}

	e.Reset(context.Background(), t)
	return e
}

// SetupPool builds the API image, starts the shared MariaDB, and provisions the
// environment pool concurrently. The returned func tears everything down.
func SetupPool(ctx context.Context) (*Pool, func(), error) {
	repoRoot, err := repoRoot()
	if err != nil {
		return nil, nil, err
	}
	sourceURL := "file://" + filepath.Join(repoRoot, "db", "migrations")

	if err := buildAPIImage(ctx, repoRoot); err != nil {
		return nil, nil, err
	}
	net, err := newNetwork(ctx)
	if err != nil {
		return nil, nil, err
	}
	mariadb, mappedPort, err := startMariaDB(ctx, net.Name)
	if err != nil {
		return nil, nil, err
	}
	rootDB, err := sql.Open("mysql", fmt.Sprintf("root:%s@tcp(127.0.0.1:%s)/", dbRootPass, mappedPort))
	if err != nil {
		return nil, nil, err
	}

	size := poolSize()
	envs := make([]*Environment, size)

	g, gctx := errgroup.WithContext(ctx)
	for i := 0; i < size; i++ {
		i := i
		g.Go(func() error {
			dbName := fmt.Sprintf("portmaster_%d", i)
			if _, err := rootDB.ExecContext(gctx, "CREATE DATABASE IF NOT EXISTS "+dbName); err != nil {
				return fmt.Errorf("create %s: %w", dbName, err)
			}

			dsn := migrateURL(mappedPort, dbName)
			if err := migrateUp(sourceURL, dsn); err != nil {
				return err
			}

			container, baseURL, err := startAPI(gctx, net.Name, dbName)
			if err != nil {
				return err
			}
			envDB, err := sql.Open("mysql", fmt.Sprintf("root:%s@tcp(127.0.0.1:%s)/%s?parseTime=true&multiStatements=true", dbRootPass, mappedPort, dbName))
			if err != nil {
				return err
			}

			envs[i] = &Environment{
				BaseURL:    baseURL,
				DB:         envDB,
				sourceURL:  sourceURL,
				migrateDSN: dsn,
				container:  container,
				fresh:      true,
			}
			return nil
		})
	}
	if err := g.Wait(); err != nil {
		return nil, nil, err
	}
	ch := make(chan *Environment, size)
	for _, e := range envs {
		ch <- e
	}

	teardown := func() {
		bg := context.Background()
		for _, e := range envs {
			if e == nil {
				continue
			}
			_ = e.DB.Close()
			_ = e.container.Terminate(bg)
		}
		_ = rootDB.Close()
		_ = mariadb.Terminate(bg)
		_ = net.Remove(bg)
	}

	return &Pool{ch: ch}, teardown, nil
}

// defaultPoolSize is one environment per story, which is as many as can ever be
// in use at once: the stories are the only parallel tests, and a leased
// environment goes back to the pool when its story ends.
//
// It used to be GOMAXPROCS, and on an 8-core machine that provisioned eight
// environments for three stories. The five spares were not free — measured on
// this suite, they cost 17s to create and a further 28s to destroy, which was
// 41% of the whole run (122.6s against 72.5s). Teardown was the larger half and
// the easier one to miss, because nothing timed it.
//
// A fourth story does not break: it blocks until one of the three frees up, and
// the run is slower rather than wrong. Bump this, or set INTEGRATION_POOL_SIZE,
// when that trade stops being worth it.
const defaultPoolSize = 3

// poolSize is the number of {API + database} environments, overridable via
// INTEGRATION_POOL_SIZE.
//
// Capped by GOMAXPROCS: more environments than the runtime will schedule tests
// on is provisioning nobody can use.
func poolSize() int {
	if v := os.Getenv("INTEGRATION_POOL_SIZE"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	if n := runtime.GOMAXPROCS(0); n < defaultPoolSize {
		return max(n, 1)
	}
	return defaultPoolSize
}

func repoRoot() (string, error) {
	_, file, _, ok := runtime.Caller(0)
	if !ok {
		return "", fmt.Errorf("cannot locate harness source")
	}
	// this file: <repo>/tests/integration/internal/harness/harness.go
	return filepath.Abs(filepath.Join(filepath.Dir(file), "..", "..", "..", ".."))
}
