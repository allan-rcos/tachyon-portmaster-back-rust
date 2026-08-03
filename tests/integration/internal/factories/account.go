package factories

import (
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// AccountUpdate builds a PUT /account body.
func AccountUpdate(name, email string) []byte {
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	emailOff := b.CreateString(email)
	fbs.AccountUpdateRequestStart(b)
	fbs.AccountUpdateRequestAddName(b, nameOff)
	fbs.AccountUpdateRequestAddEmail(b, emailOff)
	b.Finish(fbs.AccountUpdateRequestEnd(b))
	return b.FinishedBytes()
}

// PasswordChange builds a PUT /account/password body — the self-service change,
// which carries the current password, unlike PasswordReset.
func PasswordChange(current, next string) []byte {
	b := flatbuffers.NewBuilder(0)
	curOff := b.CreateString(current)
	newOff := b.CreateString(next)
	fbs.AccountPasswordChangeRequestStart(b)
	fbs.AccountPasswordChangeRequestAddCurrentPassword(b, curOff)
	fbs.AccountPasswordChangeRequestAddNewPassword(b, newOff)
	b.Finish(fbs.AccountPasswordChangeRequestEnd(b))
	return b.FinishedBytes()
}
