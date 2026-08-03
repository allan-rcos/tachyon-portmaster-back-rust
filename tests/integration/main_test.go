package integration

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"testing"

	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/client"
	"portmaster/tests/integration/internal/harness"
)

// pool is the shared set of isolated {API + database} environments, built once
// for the whole package and torn down when the run finishes.
var pool *harness.Pool

func TestMain(m *testing.M) {
	ctx := context.Background()

	p, teardown, err := harness.SetupPool(ctx)
	if err != nil {
		fmt.Fprintf(os.Stderr, "integration setup failed: %v\n", err)
		os.Exit(1)
	}
	pool = p

	code := m.Run()

	teardown()
	os.Exit(code)
}

// adminSession leases a fresh, reset environment and returns a client already
// authenticated as its bootstrap admin.
//
// The admin is created through POST /setup, which also logs them in — so no
// separate /auth/login is needed here, and the bootstrap path is exercised by
// every single test rather than by one that remembers to.
func adminSession(t *testing.T) (*harness.Environment, *client.Client) {
	t.Helper()
	env := pool.Lease(t)
	c := client.New(env.BaseURL)
	client.Setup(t, c, harness.AdminName, harness.AdminEmail, harness.AdminPassword)
	return env, c
}

// requireNoContent fails the test unless the response is exactly 204.
func requireNoContent(t *testing.T, resp client.Response) {
	t.Helper()
	if resp.Status != http.StatusNoContent {
		t.Fatalf("expected 204, got %d (body %d bytes)", resp.Status, len(resp.Body))
	}
}

// decodeRoot reads a FlatBuffers root table out of a response body, failing the
// test on a body too short to hold one.
//
// The generated GetRootAs* helpers index straight into the buffer and *panic* on
// a short read — and a panic takes down the whole test binary, so one test
// disagreeing with the contract cancels every other test running in parallel.
// This turns that into an ordinary failure of the test that caused it.
func decodeRoot[T any](t *testing.T, body []byte, root func([]byte, flatbuffers.UOffsetT) *T) *T {
	t.Helper()
	if len(body) < 8 {
		t.Fatalf("response too short to be a FlatBuffers root: %d bytes", len(body))
	}
	return root(body, 0)
}

// requireOK fails the test unless the response is a 2xx.
func requireOK(t *testing.T, resp client.Response) client.Response {
	t.Helper()
	if resp.Status < 200 || resp.Status >= 300 {
		t.Fatalf("expected 2xx, got %d (body %d bytes)", resp.Status, len(resp.Body))
	}
	return resp
}
