package element

const Doc = `
// Package {{.PackageName}} contains field arithmetic operations for modulus = 0x{{shorten .ModulusHex}}.
// 
// The API is similar to math/big (big.Int), but the operations are significantly faster (up to 20x for the modular multiplication on amd64, see also https://hackmd.io/@gnark/modular_multiplication)
// 
// The modulus is hardcoded in all the operations.
// 
// Field elements are represented as an array, and assumed to be in Montgomery form in all methods:
// 	type {{.ElementName}} [{{.NbWords}}]uint64
//
// Usage
//
// Example API signature:
// 	// Mul z = x * y (mod q)
// 	func (z *Element) Mul(x, y *Element) *Element
//
// and can be used like so:
// 	var a, b Element
// 	a.SetUint64(2)
// 	b.SetString("984896738")
// 	a.Mul(a, b)
// 	a.Sub(a, a)
// 	 .Add(a, b)
// 	 .Inv(a)
// 	b.Exp(b, new(big.Int).SetUint64(42))
//
// Modulus q =
//
// 	q[base10] = {{.Modulus}}
// 	q[base16] = 0x{{.ModulusHex}}
//
// Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package {{.PackageName}}
`
