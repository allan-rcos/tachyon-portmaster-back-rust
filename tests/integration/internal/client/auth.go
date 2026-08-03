package client

import (
	"net/http"
	"testing"

	"portmaster/tests/integration/internal/factories"
	"portmaster/tests/integration/internal/fbs"
)

// Setup performs a FlatBuffers POST /setup, creating the first user of a fresh
// environment and logging them in. The session cookies are retained by the
// client's jar for later calls.
//
// This is the only way a reset environment gets a user: every other route that
// creates one sits behind a permission that nobody holds yet.
func Setup(t *testing.T, c *Client, name, email, password string) *fbs.LoginResponse {
	t.Helper()

	resp := c.Post(t, "/setup", factories.Setup(name, email, password))
	if resp.Status != http.StatusCreated {
		t.Fatalf("setup as %s: status %d", email, resp.Status)
	}

	return loginResponse(t, resp)
}

// LoginAs performs a FlatBuffers /auth/login and returns the decoded response.
// The auth/refresh cookies are retained by the client's jar for later calls.
func LoginAs(t *testing.T, c *Client, email, password string) *fbs.LoginResponse {
	t.Helper()

	resp := c.Post(t, "/auth/login", factories.Login(email, password))
	if resp.Status != http.StatusOK {
		t.Fatalf("login as %s: status %d", email, resp.Status)
	}

	return loginResponse(t, resp)
}

// loginResponse decodes a session body, failing rather than panicking when the
// response is too short to hold a FlatBuffers root — a panic here would take the
// whole test binary down with it.
func loginResponse(t *testing.T, resp Response) *fbs.LoginResponse {
	t.Helper()
	if len(resp.Body) < 8 {
		t.Fatalf("session response too short to be a FlatBuffers root: %d bytes", len(resp.Body))
	}
	return fbs.GetRootAsLoginResponse(resp.Body, 0)
}
