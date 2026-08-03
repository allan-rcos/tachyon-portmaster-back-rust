package factories

import (
	"fmt"

	"github.com/brianvoe/gofakeit/v7"
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// Container is a generated container-create payload plus its values.
type Container struct {
	Code        string
	MaxCapacity float64
	Bytes       []byte
}

// NewContainer builds a POST /containers body with a unique code.
//
// The random suffix matters: container codes are unique across the whole
// database, and stories run in parallel against a shared image, so a fixed code
// would make one story's success depend on another story's timing.
func NewContainer() Container {
	code := fmt.Sprintf("CT-%s", gofakeit.LetterN(8))
	capacity := 1000.0

	b := flatbuffers.NewBuilder(0)
	codeOff := b.CreateString(code)
	fbs.ContainerCreateRequestStart(b)
	fbs.ContainerCreateRequestAddCode(b, codeOff)
	fbs.ContainerCreateRequestAddMaxCapacity(b, capacity)
	b.Finish(fbs.ContainerCreateRequestEnd(b))

	return Container{Code: code, MaxCapacity: capacity, Bytes: b.FinishedBytes()}
}

// ContainerUpdate builds a PUT /containers/{id} body.
func ContainerUpdate(maxCapacity float64) []byte {
	b := flatbuffers.NewBuilder(0)
	fbs.ContainerUpdateRequestStart(b)
	fbs.ContainerUpdateRequestAddMaxCapacity(b, maxCapacity)
	b.Finish(fbs.ContainerUpdateRequestEnd(b))
	return b.FinishedBytes()
}

// ContainerWithCode builds a POST /containers body reusing a known code, to
// exercise the uniqueness rule.
func ContainerWithCode(code string, capacity float64) []byte {
	b := flatbuffers.NewBuilder(0)
	codeOff := b.CreateString(code)
	fbs.ContainerCreateRequestStart(b)
	fbs.ContainerCreateRequestAddCode(b, codeOff)
	fbs.ContainerCreateRequestAddMaxCapacity(b, capacity)
	b.Finish(fbs.ContainerCreateRequestEnd(b))
	return b.FinishedBytes()
}
