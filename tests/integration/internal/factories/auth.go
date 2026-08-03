package factories

import (
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// Login builds a POST /auth/login body.
func Login(email, password string) []byte {
	b := flatbuffers.NewBuilder(0)
	emailOff := b.CreateString(email)
	passOff := b.CreateString(password)
	fbs.LoginRequestStart(b)
	fbs.LoginRequestAddEmail(b, emailOff)
	fbs.LoginRequestAddPassword(b, passOff)
	b.Finish(fbs.LoginRequestEnd(b))
	return b.FinishedBytes()
}

// Setup builds a POST /setup body.
//
// The client package wraps this in a helper that sends it and asserts a 201,
// which is what most callers want; this bare form is for the tests that need to
// drive the response themselves — asserting the 409 the second caller gets, for
// instance.
func Setup(name, email, password string) []byte {
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	emailOff := b.CreateString(email)
	passOff := b.CreateString(password)
	fbs.SetupRequestStart(b)
	fbs.SetupRequestAddName(b, nameOff)
	fbs.SetupRequestAddEmail(b, emailOff)
	fbs.SetupRequestAddPassword(b, passOff)
	b.Finish(fbs.SetupRequestEnd(b))
	return b.FinishedBytes()
}
