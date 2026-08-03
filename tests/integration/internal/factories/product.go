package factories

import (
	"github.com/brianvoe/gofakeit/v7"
	flatbuffers "github.com/google/flatbuffers/go"

	"portmaster/tests/integration/internal/fbs"
)

// Product is a generated product-create payload plus the values it carries.
type Product struct {
	Name      string
	Density   float64
	RiskClass fbs.RiskClass
	Bytes     []byte
}

// NewProduct builds a POST /products body.
func NewProduct() Product {
	name := gofakeit.Sentence(2)
	density := gofakeit.Float64Range(0.1, 5.0)
	risk := fbs.RiskClassClass3FlammableLiquids

	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	fbs.ProductCreateRequestStart(b)
	fbs.ProductCreateRequestAddName(b, nameOff)
	fbs.ProductCreateRequestAddDensity(b, density)
	fbs.ProductCreateRequestAddRiskClass(b, risk)
	b.Finish(fbs.ProductCreateRequestEnd(b))

	return Product{Name: name, Density: density, RiskClass: risk, Bytes: b.FinishedBytes()}
}

// ProductUpdate builds a PUT /products/{id} body, returning the new name so the
// caller can assert the server stored it.
func ProductUpdate() (name string, body []byte) {
	name = gofakeit.Sentence(2)
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString(name)
	fbs.ProductUpdateRequestStart(b)
	fbs.ProductUpdateRequestAddName(b, nameOff)
	fbs.ProductUpdateRequestAddDensity(b, gofakeit.Float64Range(0.1, 5.0))
	fbs.ProductUpdateRequestAddRiskClass(b, fbs.RiskClassClass2Gases)
	b.Finish(fbs.ProductUpdateRequestEnd(b))
	return name, b.FinishedBytes()
}

// InvalidProduct builds a POST /products body with a blank name.
//
// The buffer itself is well-formed, so the rejection has to come from the
// domain's non-empty-name rule rather than from the wire — which is the only
// way to prove that rule is the thing enforcing it.
func InvalidProduct() []byte {
	b := flatbuffers.NewBuilder(0)
	nameOff := b.CreateString("")
	fbs.ProductCreateRequestStart(b)
	fbs.ProductCreateRequestAddName(b, nameOff)
	fbs.ProductCreateRequestAddDensity(b, 1.0)
	fbs.ProductCreateRequestAddRiskClass(b, fbs.RiskClassClass3FlammableLiquids)
	b.Finish(fbs.ProductCreateRequestEnd(b))
	return b.FinishedBytes()
}
