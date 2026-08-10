package factories

import (
	"github.com/brianvoe/gofakeit/v7"
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// Password returns a password that satisfies the domain policy by construction:
// at least eight characters, with a lower-case letter, an upper-case letter and
// a digit.
//
// gofakeit.Password draws every character from the union of the classes it was
// asked for and guarantees none of them individually, so roughly 3% of
// twelve-character draws come back missing one — and the UserTM answers those
// with a 422. Every story that creates a user went through here, which is what
// made them fail a few runs in a hundred for a reason that had nothing to do
// with what they were testing. Drawing the three required characters explicitly
// and shuffling them into the rest keeps the password unpredictable without
// leaving the policy to chance.
func Password() string {
	chars := []byte(gofakeit.Password(true, true, true, false, false, 9))
	chars = append(chars,
		byte('a'+gofakeit.Number(0, 25)),
		byte('A'+gofakeit.Number(0, 25)),
		byte('0'+gofakeit.Number(0, 9)),
	)

	// Fisher-Yates, so the guaranteed three are not always the last three.
	for i := len(chars) - 1; i > 0; i-- {
		j := gofakeit.Number(0, i)
		chars[i], chars[j] = chars[j], chars[i]
	}

	return string(chars)
}

// User is a generated user-create payload plus its values.
type User struct {
	Name     string
	Email    string
	Password string
	RoleIDs  []string
	Bytes    []byte
}

// NewUser builds a POST /users body assigning the given role ids.
func NewUser(roleIDs ...string) User {
	name := gofakeit.Name()
	email := gofakeit.Email()
	password := Password()

	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	emailOff := b.CreateString(email)
	passOff := b.CreateString(password)
	roleOffsets := make([]flatbuffers.UOffsetT, len(roleIDs))
	for i, r := range roleIDs {
		roleOffsets[i] = b.CreateString(r)
	}
	fbs.UserCreateRequestStartRoleIdsVector(b, len(roleOffsets))
	for i := len(roleOffsets) - 1; i >= 0; i-- {
		b.PrependUOffsetT(roleOffsets[i])
	}
	rolesVec := b.EndVector(len(roleOffsets))

	fbs.UserCreateRequestStart(b)
	fbs.UserCreateRequestAddName(b, nameOff)
	fbs.UserCreateRequestAddEmail(b, emailOff)
	fbs.UserCreateRequestAddInitialPassword(b, passOff)
	fbs.UserCreateRequestAddRoleIds(b, rolesVec)
	b.Finish(fbs.UserCreateRequestEnd(b))

	return User{Name: name, Email: email, Password: password, RoleIDs: roleIDs, Bytes: b.FinishedBytes()}
}

// UserUpdate builds a PUT /users/{id} body.
func UserUpdate(name, email string) []byte {
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	emailOff := b.CreateString(email)
	fbs.UserUpdateRequestStart(b)
	fbs.UserUpdateRequestAddName(b, nameOff)
	fbs.UserUpdateRequestAddEmail(b, emailOff)
	b.Finish(fbs.UserUpdateRequestEnd(b))
	return b.FinishedBytes()
}

// PasswordReset builds a PUT /users/{id}/password body — the administrator's
// reset, which needs no current password, unlike PasswordChange.
func PasswordReset(newPassword string) []byte {
	b := flatbuffers.NewBuilder(0)
	pwOff := b.CreateString(newPassword)
	fbs.UserAdminPasswordResetRequestStart(b)
	fbs.UserAdminPasswordResetRequestAddNewPassword(b, pwOff)
	b.Finish(fbs.UserAdminPasswordResetRequestEnd(b))
	return b.FinishedBytes()
}

// RoleIDs builds the body for PUT /users/{id}/roles.
//
// This marshalled an inline JSON object until UserRolesUpdateRequest entered the
// contract: the endpoint was the last body in the API with no table of its own.
// Sending the format the rest of the suite negotiates got a 400 — and, worse,
// whatever the hand-rolled parser could not read became an empty list, which the
// use case reads as "revoke every role". The factory now speaks the same wire as
// every other body here.
func RoleIDs(ids ...string) []byte {
	b := flatbuffers.NewBuilder(0)

	roleOffsets := make([]flatbuffers.UOffsetT, len(ids))
	for i, id := range ids {
		roleOffsets[i] = b.CreateString(id)
	}
	fbs.UserRolesUpdateRequestStartRoleIdsVector(b, len(roleOffsets))
	for i := len(roleOffsets) - 1; i >= 0; i-- {
		b.PrependUOffsetT(roleOffsets[i])
	}
	rolesVec := b.EndVector(len(roleOffsets))

	fbs.UserRolesUpdateRequestStart(b)
	fbs.UserRolesUpdateRequestAddRoleIds(b, rolesVec)
	b.Finish(fbs.UserRolesUpdateRequestEnd(b))

	return b.FinishedBytes()
}

// UserWithEmail builds a POST /users body with a chosen e-mail and password, to
// exercise the uniqueness and password-strength rules.
func UserWithEmail(email, password string) []byte {
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(gofakeit.Name())
	emailOff := b.CreateString(email)
	passOff := b.CreateString(password)
	fbs.UserCreateRequestStartRoleIdsVector(b, 0)
	rolesVec := b.EndVector(0)
	fbs.UserCreateRequestStart(b)
	fbs.UserCreateRequestAddName(b, nameOff)
	fbs.UserCreateRequestAddEmail(b, emailOff)
	fbs.UserCreateRequestAddInitialPassword(b, passOff)
	fbs.UserCreateRequestAddRoleIds(b, rolesVec)
	b.Finish(fbs.UserCreateRequestEnd(b))
	return b.FinishedBytes()
}
