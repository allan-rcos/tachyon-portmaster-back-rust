package factories

import (
	"fmt"

	"github.com/brianvoe/gofakeit/v7"
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// Role is a generated role-create payload plus its values.
type Role struct {
	Name        string
	Permissions []string
	Bytes       []byte
}

// NewRole builds a POST /roles body carrying the given permission slugs.
func NewRole(permissions ...string) Role {
	name := fmt.Sprintf("%s-%s", gofakeit.JobTitle(), gofakeit.LetterN(4))

	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	permOffsets := make([]flatbuffers.UOffsetT, len(permissions))
	for i, p := range permissions {
		permOffsets[i] = b.CreateString(p)
	}
	fbs.RoleCreateRequestStartPermissionsVector(b, len(permOffsets))
	for i := len(permOffsets) - 1; i >= 0; i-- {
		b.PrependUOffsetT(permOffsets[i])
	}
	permsVec := b.EndVector(len(permOffsets))

	fbs.RoleCreateRequestStart(b)
	fbs.RoleCreateRequestAddName(b, nameOff)
	fbs.RoleCreateRequestAddPermissions(b, permsVec)
	b.Finish(fbs.RoleCreateRequestEnd(b))

	return Role{Name: name, Permissions: permissions, Bytes: b.FinishedBytes()}
}

// RolePermissions builds a PUT /roles/{id}/permissions body.
//
// The endpoint replaces the whole set rather than merging into it, so the slugs
// passed here are the role's permissions afterwards — omitting one revokes it.
func RolePermissions(permissions ...string) []byte {
	b := flatbuffers.NewBuilder(0)
	permOffsets := make([]flatbuffers.UOffsetT, len(permissions))
	for i, p := range permissions {
		permOffsets[i] = b.CreateString(p)
	}
	fbs.RolePermissionsUpdateRequestStartPermissionsVector(b, len(permOffsets))
	for i := len(permOffsets) - 1; i >= 0; i-- {
		b.PrependUOffsetT(permOffsets[i])
	}
	vec := b.EndVector(len(permOffsets))
	fbs.RolePermissionsUpdateRequestStart(b)
	fbs.RolePermissionsUpdateRequestAddPermissions(b, vec)
	b.Finish(fbs.RolePermissionsUpdateRequestEnd(b))
	return b.FinishedBytes()
}
