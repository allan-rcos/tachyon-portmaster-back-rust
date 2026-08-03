// Package factories builds FlatBuffers request payloads for the integration
// suite, filling them with fake-but-plausible data (gofakeit).
//
// Factories come in two shapes, and which one to use follows from what the test
// asserts:
//
//   - those returning a struct (Product, Container, Role, User) carry both the
//     encoded Bytes and the values that went into them, so a test can create a
//     resource and then assert the server echoed those values back;
//   - those returning a bare []byte are for requests whose values the test
//     already holds, or does not check.
//
// One file per feature, mirroring the layering of src/: a payload for a route
// under /products lives in product.go. Negative payloads — well-formed
// FlatBuffers that a domain rule must reject — sit next to the valid factory
// for the same feature rather than in a shared bucket, so adding a rule and
// adding its counter-example land in the same file.
package factories
