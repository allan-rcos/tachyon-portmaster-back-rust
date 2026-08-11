package integration

import (
	"net/http"
	"testing"

	"github.com/brianvoe/gofakeit/v7"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"portmaster/tests/integration/internal/client"
	"portmaster/tests/integration/internal/factories"
	"portmaster/tests/integration/internal/fbs"
	"portmaster/tests/integration/internal/harness"
)

// Cookie names, as configured for the integration environments.
const (
	authCookie    = "auth_token"
	refreshCookie = "refresh_token"
)

// TestSessionStory walks one deployment's whole session life: bootstrapping it,
// signing in and out, rotating credentials, and the ways each of those is
// refused.
//
// It is a story rather than a test per endpoint because leasing an environment
// costs a schema reset and a server restart — around twenty seconds. Spending
// that per assertion would buy nothing: these steps share one system and read
// naturally in sequence, and running them together means the *order* is tested
// too, which is where session bugs actually live (a token that still works
// after logout, a rotation that outlives its predecessor).
//
// Sub-tests deliberately do not run in parallel: each depends on the state the
// previous one left behind.
func TestSessionStory(t *testing.T) {
	t.Parallel()
	env := pool.Lease(t)
	c := client.New(env.BaseURL)

	t.Run("info is served before anyone has signed in", func(t *testing.T) {
		info := decodeRoot(t, requireOK(t, c.Get(t, "/info")).Body, fbs.GetRootAsProjectInfo)
		assert.NotEmpty(t, string(info.Name()))
	})

	// The unversioned path is a convenience that follows whichever published
	// version still serves a route; /v1 is the address a client should pin to,
	// and is what swagger.json advertises. Both must answer, and identically.
	t.Run("a route answers under its version prefix too", func(t *testing.T) {
		versioned := decodeRoot(t, requireOK(t, c.Get(t, "/v1/info")).Body, fbs.GetRootAsProjectInfo)
		root := decodeRoot(t, requireOK(t, c.Get(t, "/info")).Body, fbs.GetRootAsProjectInfo)

		assert.Equal(t, string(root.Name()), string(versioned.Name()))
		assert.Equal(t, string(root.Version()), string(versioned.Version()))
	})

	t.Run("setup creates the first user and then refuses to run again", func(t *testing.T) {
		created := client.Setup(t, c, harness.AdminName, harness.AdminEmail, harness.AdminPassword)
		require.NotEmpty(t, created.Token(), "setup should sign the operator in")
		assert.Equal(t, harness.AdminEmail, string(created.User(nil).Email()))
		assert.NotEmpty(t, c.Cookie(t, refreshCookie))

		other := client.New(env.BaseURL)
		resp := other.Post(t, "/setup", factories.Setup("Second", "second@portmaster.local", "Portmaster2"))
		assert.Equal(t, http.StatusConflict, resp.Status,
			"the door closes behind the first user, whoever asks next")
	})

	t.Run("login refuses bad credentials and accepts the real ones", func(t *testing.T) {
		wrongPassword := c.Post(t, "/auth/login", factories.Login(harness.AdminEmail, "Wrong-Pass99"))
		assert.Equal(t, http.StatusUnauthorized, wrongPassword.Status)

		unknownEmail := c.Post(t, "/auth/login", factories.Login("nobody@portmaster.local", harness.AdminPassword))
		assert.Equal(t, http.StatusUnauthorized, unknownEmail.Status,
			"an unknown e-mail must be indistinguishable from a wrong password")

		session := client.LoginAs(t, c, harness.AdminEmail, harness.AdminPassword)
		assert.Equal(t, harness.AdminEmail, string(session.User(nil).Email()))
	})

	t.Run("refresh rejects anything that is not a live refresh token", func(t *testing.T) {
		issued := c.Cookie(t, refreshCookie)
		require.NotEmpty(t, issued)

		anonymous := client.New(env.BaseURL)
		assert.Equal(t, http.StatusUnauthorized, anonymous.Post(t, "/auth/refresh", nil).Status,
			"no refresh cookie at all")

		// Both tokens are signed with the same key, so only the `typ` claim
		// separates them.
		access := c.Cookie(t, authCookie)
		require.NotEmpty(t, access)
		c.SetCookie(t, refreshCookie, access)
		assert.Equal(t, http.StatusUnauthorized, c.Post(t, "/auth/refresh", nil).Status,
			"an access token presented as a refresh token")

		c.SetCookie(t, refreshCookie, "not-a-token")
		assert.Equal(t, http.StatusUnauthorized, c.Post(t, "/auth/refresh", nil).Status,
			"a value we never signed")

		c.SetCookie(t, refreshCookie, issued)
	})

	t.Run("refresh rotates, and the token it consumed never works again", func(t *testing.T) {
		spent := c.Cookie(t, refreshCookie)
		require.NotEmpty(t, spent)

		requireNoContent(t, c.Post(t, "/auth/refresh", nil))
		assert.NotEqual(t, spent, c.Cookie(t, refreshCookie), "refresh must rotate the token")

		// Still validly signed and nowhere near expiry — only the marker knows it
		// was spent, which is the marker's whole job.
		c.SetCookie(t, refreshCookie, spent)
		assert.Equal(t, http.StatusUnauthorized, c.Post(t, "/auth/refresh", nil).Status,
			"a consumed refresh token must not work twice")
	})

	t.Run("logout revokes the live token", func(t *testing.T) {
		// Re-establish a clean session: the previous step left a consumed token
		// in the jar.
		client.LoginAs(t, c, harness.AdminEmail, harness.AdminPassword)
		revoked := c.Cookie(t, refreshCookie)
		require.NotEmpty(t, revoked)

		requireNoContent(t, c.Post(t, "/auth/logout", nil))

		c.SetCookie(t, refreshCookie, revoked)
		assert.Equal(t, http.StatusUnauthorized, c.Post(t, "/auth/refresh", nil).Status,
			"a revoked refresh token must not be usable")
	})

	t.Run("the account survives its own sessions ending", func(t *testing.T) {
		// Logging out revokes a token, never the account.
		client.LoginAs(t, c, harness.AdminEmail, harness.AdminPassword)

		profile := decodeRoot(t, requireOK(t, c.Get(t, "/account")).Body, fbs.GetRootAsAccountProfileResponse)
		assert.Equal(t, harness.AdminEmail, string(profile.Email()))
	})

	t.Run("changing the password invalidates the old one", func(t *testing.T) {
		const newPassword = "Renewed-Pass99"

		newName, newEmail := gofakeit.Name(), gofakeit.Email()
		requireOK(t, c.Put(t, "/account", factories.AccountUpdate(newName, newEmail)))

		wrongCurrent := c.Put(t, "/account/password", factories.PasswordChange("Not-The-Current1", newPassword))
		assert.GreaterOrEqual(t, wrongCurrent.Status, 400,
			"changing a password must require the current one")

		weak := c.Put(t, "/account/password", factories.PasswordChange(harness.AdminPassword, "weak"))
		assert.Equal(t, http.StatusUnprocessableEntity, weak.Status,
			"the password policy applies to changes, not just to creation")

		requireOK(t, c.Put(t, "/account/password", factories.PasswordChange(harness.AdminPassword, newPassword)))

		// The e-mail moved in the same story, so the new one is the identity now.
		fresh := client.New(env.BaseURL)
		assert.Equal(t, http.StatusUnauthorized,
			fresh.Post(t, "/auth/login", factories.Login(newEmail, harness.AdminPassword)).Status,
			"the replaced password must stop working")

		session := client.LoginAs(t, fresh, newEmail, newPassword)
		assert.Equal(t, newEmail, string(session.User(nil).Email()))
	})
}
