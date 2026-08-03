package integration

import (
	"net/http"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"portmaster/tests/integration/internal/client"
	"portmaster/tests/integration/internal/factories"
	"portmaster/tests/integration/internal/fbs"
)

// TestAdministrationStory covers granting and revoking access: creating roles,
// handing them to users, and what happens to a user once they exist.
//
// The pieces only mean anything together — a role is worth creating because a
// user gets it, and a permission is worth granting because it decides what that
// user may do. The last step spends the role it created by signing in as the
// user and watching authorization actually bite, which no single-endpoint test
// could reach.
func TestAdministrationStory(t *testing.T) {
	t.Parallel()
	env, c := adminSession(t)

	var (
		roleID string
		userID string
		user   factories.User
	)

	t.Run("the permission catalogue is what a role can be built from", func(t *testing.T) {
		list := decodeRoot(t, requireOK(t, c.Get(t, "/metadata/permissions")).Body, fbs.GetRootAsPermissionListResponse)

		slugs := permissionSlugs(t, list)
		require.NotEmpty(t, slugs, "every guarded use case declares its permission at WorkerStart")

		// The catalogue stopped being an enum a client could read off the
		// schema, so this is the only place the grantable set exists. Asserting
		// on slugs the rest of this story goes on to grant is what ties the
		// listing to the guard that consumes it.
		assert.Contains(t, slugs, "permission:list", "the listing's own permission is in the catalogue it lists")
		assert.Contains(t, slugs, "role:list")
		assert.Contains(t, slugs, "product:read")
		assert.Contains(t, slugs, "metrics:read")

		for _, slug := range slugs {
			assert.Contains(t, slug, ":", "a slug is <resource>:<action>: %q", slug)
		}

		filtered := decodeRoot(t, requireOK(t, c.Get(t, "/metadata/permissions?search=product")).Body, fbs.GetRootAsPermissionListResponse)
		require.Positive(t, filtered.DataLength())
		assert.Less(t, filtered.DataLength(), list.DataLength(), "a search term must actually narrow the catalogue")
		for _, slug := range permissionSlugs(t, filtered) {
			assert.Contains(t, slug, "product", "every row must match the term: %q", slug)
		}

		// Free text is normalised the same way everywhere, so case is not
		// something the caller has to get right.
		upper := decodeRoot(t, requireOK(t, c.Get(t, "/metadata/permissions?search=PRODUCT")).Body, fbs.GetRootAsPermissionListResponse)
		assert.Equal(t, filtered.DataLength(), upper.DataLength())

		none := decodeRoot(t, requireOK(t, c.Get(t, "/metadata/permissions?search=nothing-declares-this")).Body, fbs.GetRootAsPermissionListResponse)
		assert.Zero(t, none.DataLength(), "no match is an empty array, not a 404")
	})

	t.Run("roles are created and their permissions replaced wholesale", func(t *testing.T) {
		role := factories.NewRole("product:read")
		created := decodeRoot(t, requireOK(t, c.Post(t, "/roles", role.Bytes)).Body, fbs.GetRootAsRoleResponse)
		roleID = string(created.Id())
		require.NotEmpty(t, roleID)
		assert.Equal(t, role.Name, string(created.Name()))

		list := decodeRoot(t, requireOK(t, c.Get(t, "/roles")).Body, fbs.GetRootAsRoleListResponse)
		assert.GreaterOrEqual(t, list.Total(), int32(1))

		// There is no GET /roles/{id} in the contract, so the result is read back
		// off the listing.
		requireOK(t, c.Put(t, "/roles/"+roleID+"/permissions",
			factories.RolePermissions("product:read", "metrics:read")))

		after := decodeRoot(t, requireOK(t, c.Get(t, "/roles")).Body, fbs.GetRootAsRoleListResponse)
		found := false
		for i := 0; i < after.DataLength(); i++ {
			var role fbs.RoleResponse
			require.True(t, after.Data(&role, i))
			if string(role.Id()) == roleID {
				found = true
				assert.Equal(t, 2, role.PermissionsLength(), "permissions are replaced, not merged")
			}
		}
		assert.True(t, found, "the created role must appear in the listing")
	})

	t.Run("users are created, and duplicates and weak passwords refused", func(t *testing.T) {
		user = factories.NewUser(roleID)
		created := decodeRoot(t, requireOK(t, c.Post(t, "/users", user.Bytes)).Body, fbs.GetRootAsUserAdminResponse)
		userID = string(created.Id())
		require.NotEmpty(t, userID)
		assert.Equal(t, user.Email, string(created.Email()))

		assert.Equal(t, http.StatusConflict,
			c.Post(t, "/users", factories.UserWithEmail(user.Email, "Another-Pass9")).Status,
			"an e-mail already in use must be a conflict, not a 500")

		assert.Equal(t, http.StatusUnprocessableEntity,
			c.Post(t, "/users", factories.UserWithEmail("fresh@portmaster.local", "weak")).Status,
			"the password policy applies to admin-created users too")

		assert.Equal(t, http.StatusNotFound, c.Get(t, "/users/P0000000").Status)

		list := decodeRoot(t, requireOK(t, c.Get(t, "/users")).Body, fbs.GetRootAsUserListResponse)
		assert.GreaterOrEqual(t, list.DataLength(), 2, "the bootstrap admin plus the new user")
	})

	t.Run("an administrator can rename a user, reset their password and re-assign roles", func(t *testing.T) {
		requireOK(t, c.Put(t, "/users/"+userID, factories.UserUpdate("Renamed User", user.Email)))

		renamed := decodeRoot(t, requireOK(t, c.Get(t, "/users/"+userID)).Body, fbs.GetRootAsUserAdminResponse)
		assert.Equal(t, "Renamed User", string(renamed.Name()))

		assert.Equal(t, http.StatusUnprocessableEntity,
			c.Put(t, "/users/"+userID+"/password", factories.PasswordReset("weak")).Status,
			"a reset is still a password, and the policy still applies")

		requireOK(t, c.Put(t, "/users/"+userID+"/password", factories.PasswordReset("Reset-Pass123")))
		requireOK(t, c.Put(t, "/users/"+userID+"/roles", factories.RoleIDs(roleID)))
	})

	t.Run("the granted permissions are exactly what the user may do", func(t *testing.T) {
		// The role granted above holds product:read and metrics:read, nothing else.
		asUser := client.New(env.BaseURL)
		client.LoginAs(t, asUser, user.Email, "Reset-Pass123")

		requireOK(t, asUser.Get(t, "/products"))

		// metrics:read was read off the catalogue at the top of this story, so
		// this is also what proves the slug the listing published is the same
		// string the guard compares against.
		requireOK(t, asUser.Get(t, "/metrics"))

		assert.Equal(t, http.StatusForbidden, asUser.Post(t, "/products", factories.NewProduct().Bytes).Status,
			"product:create was never granted")
		assert.Equal(t, http.StatusForbidden, asUser.Get(t, "/users").Status,
			"user:list was never granted")

		// The catalogue is metadata, not public: reading what may be granted is
		// itself a grant.
		assert.Equal(t, http.StatusForbidden, asUser.Get(t, "/metadata/permissions").Status,
			"permission:list was never granted")
	})

	t.Run("deleting a user ends their access", func(t *testing.T) {
		requireOK(t, c.Delete(t, "/users/"+userID))

		assert.Equal(t, http.StatusNotFound, c.Get(t, "/users/"+userID).Status)

		assert.Equal(t, http.StatusUnauthorized,
			client.New(env.BaseURL).Post(t, "/auth/login", factories.Login(user.Email, "Reset-Pass123")).Status,
			"a deleted user must not be able to sign in again")
	})
}

// permissionSlugs flattens a permission catalogue listing into the slugs it
// carries.
func permissionSlugs(t *testing.T, list *fbs.PermissionListResponse) []string {
	t.Helper()

	slugs := make([]string, 0, list.DataLength())
	for i := 0; i < list.DataLength(); i++ {
		var item fbs.MetadataItemResponse
		require.True(t, list.Data(&item, i))
		slugs = append(slugs, string(item.Slug()))
	}
	return slugs
}
