package factories

import (
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// LoadItem builds a POST /manifests/load-item body.
func LoadItem(containerID, productID string, quantity float64) []byte {
	b := flatbuffers.NewBuilder(0)
	cOff := b.CreateString(containerID)
	pOff := b.CreateString(productID)
	fbs.LoadItemRequestStart(b)
	fbs.LoadItemRequestAddContainerId(b, cOff)
	fbs.LoadItemRequestAddProductId(b, pOff)
	fbs.LoadItemRequestAddQuantity(b, quantity)
	b.Finish(fbs.LoadItemRequestEnd(b))
	return b.FinishedBytes()
}

// UnloadItem builds a POST /manifests/unload-item body.
func UnloadItem(containerID, productID string, quantity float64) []byte {
	b := flatbuffers.NewBuilder(0)
	cOff := b.CreateString(containerID)
	pOff := b.CreateString(productID)
	fbs.UnloadItemRequestStart(b)
	fbs.UnloadItemRequestAddContainerId(b, cOff)
	fbs.UnloadItemRequestAddProductId(b, pOff)
	fbs.UnloadItemRequestAddQuantity(b, quantity)
	b.Finish(fbs.UnloadItemRequestEnd(b))
	return b.FinishedBytes()
}
