// Package client is a thin FlatBuffers-over-HTTP driver: it sends and expects
// application/x-flatbuffers, and keeps a cookie jar so the auth/refresh cookies
// set by /auth/login ride along on subsequent requests.
package client

import (
	"bytes"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"testing"
	"time"
)

const contentType = "application/x-flatbuffers"

// Client drives one API environment over the FlatBuffers wire.
type Client struct {
	baseURL string
	http    *http.Client
}

// Response is a decoded HTTP response: status plus the raw FlatBuffers body.
type Response struct {
	Status int
	Body   []byte
}

// New returns a client for the given base URL with its own cookie jar.
func New(baseURL string) *Client {
	jar, _ := cookiejar.New(nil)
	return &Client{
		baseURL: baseURL,
		http:    &http.Client{Jar: jar, Timeout: 20 * time.Second},
	}
}

func (c *Client) do(t *testing.T, method, path string, body []byte) Response {
	t.Helper()

	var reader io.Reader
	if body != nil {
		reader = bytes.NewReader(body)
	}

	req, err := http.NewRequest(method, c.baseURL+path, reader)
	if err != nil {
		t.Fatalf("build request %s %s: %v", method, path, err)
	}
	req.Header.Set("Accept", contentType)
	if body != nil {
		req.Header.Set("Content-Type", contentType)
	}

	resp, err := c.http.Do(req)
	if err != nil {
		t.Fatalf("%s %s: %v", method, path, err)
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read body %s %s: %v", method, path, err)
	}

	return Response{Status: resp.StatusCode, Body: data}
}

// Cookie returns the value of a cookie currently held in the jar, or "" when it
// is not set. Used by tests that need to hold on to a token the server rotated
// away, in order to prove the old one no longer works.
func (c *Client) Cookie(t *testing.T, name string) string {
	t.Helper()

	u, err := url.Parse(c.baseURL)
	if err != nil {
		t.Fatalf("parse base url: %v", err)
	}

	for _, ck := range c.http.Jar.Cookies(u) {
		if ck.Name == name {
			return ck.Value
		}
	}
	return ""
}

// SetCookie overwrites a cookie in the jar, letting a test present a value the
// server would never have sent at that point — a consumed refresh token, or an
// access token in the refresh slot.
func (c *Client) SetCookie(t *testing.T, name, value string) {
	t.Helper()

	u, err := url.Parse(c.baseURL)
	if err != nil {
		t.Fatalf("parse base url: %v", err)
	}

	c.http.Jar.SetCookies(u, []*http.Cookie{{Name: name, Value: value, Path: "/"}})
}

func (c *Client) Get(t *testing.T, path string) Response {
	return c.do(t, http.MethodGet, path, nil)
}

func (c *Client) Post(t *testing.T, path string, body []byte) Response {
	return c.do(t, http.MethodPost, path, body)
}

func (c *Client) Put(t *testing.T, path string, body []byte) Response {
	return c.do(t, http.MethodPut, path, body)
}

func (c *Client) Delete(t *testing.T, path string) Response {
	return c.do(t, http.MethodDelete, path, nil)
}
