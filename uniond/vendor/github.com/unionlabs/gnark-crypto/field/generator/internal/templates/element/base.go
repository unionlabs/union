package element

const Base = `

import (
	"math/big"
	"math/bits"
	"io"
	"crypto/rand"
	"encoding/binary"
	"strconv"
	"errors"
	"reflect"
	"strings"

	"github.com/consensys/gnark-crypto/field/hash"
	"github.com/consensys/gnark-crypto/field/pool"
	"github.com/bits-and-blooms/bitset"
)

// {{.ElementName}} represents a field element stored on {{.NbWords}} words (uint64)
//
// {{.ElementName}} are assumed to be in Montgomery form in all methods.
//
// Modulus q =
//
// 	q[base10] = {{.Modulus}}
// 	q[base16] = 0x{{.ModulusHex}}
//
// Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
type {{.ElementName}} [{{.NbWords}}]uint64

const (
	Limbs = {{.NbWords}} 	// number of 64 bits words needed to represent a {{.ElementName}}
	Bits = {{.NbBits}} 		// number of bits needed to represent a {{.ElementName}}
	Bytes = {{.NbBytes}} 	// number of bytes needed to represent a {{.ElementName}}
)


// Field modulus q
const (
{{- range $i := $.NbWordsIndexesFull}}
	q{{$i}} uint64 = {{index $.Q $i}}
	{{- if eq $.NbWords 1}}
		q uint64 = q0
	{{- end}}
{{- end}}
)

var q{{.ElementName}} = {{.ElementName}}{
	{{- range $i := $.NbWordsIndexesFull}}
	q{{$i}},{{end}}
}

var _modulus big.Int 		// q stored as big.Int

// Modulus returns q as a big.Int
//
// 	q[base10] = {{.Modulus}}
// 	q[base16] = 0x{{.ModulusHex}}
func Modulus() *big.Int {
	return new(big.Int).Set(&_modulus)
}

// q + r'.r = 1, i.e., qInvNeg = - q⁻¹ mod r
// used for Montgomery reduction
const qInvNeg uint64 = {{index .QInverse 0}}

func init() {
	_modulus.SetString("{{.ModulusHex}}", 16)
}

// New{{.ElementName}} returns a new {{.ElementName}} from a uint64 value
//
// it is equivalent to
// 		var v {{.ElementName}}
// 		v.SetUint64(...)
func New{{.ElementName}}(v uint64) {{.ElementName}} {
	z := {{.ElementName}}{v}
	z.Mul(&z, &rSquare)
	return z
}

// SetUint64 sets z to v and returns z
func (z *{{.ElementName}}) SetUint64(v uint64) *{{.ElementName}} {
	//  sets z LSB to v (non-Montgomery form) and convert z to Montgomery form
	*z = {{.ElementName}}{v}
	return z.Mul(z, &rSquare) // z.toMont()
}

// SetInt64 sets z to v and returns z
func (z *{{.ElementName}}) SetInt64(v int64) *{{.ElementName}} {

	// absolute value of v
	m := v >> 63
	z.SetUint64(uint64((v ^ m) - m))

	if m != 0 {
		// v is negative
		z.Neg(z)
	}

	return z
}

// Set z = x and returns z
func (z *{{.ElementName}}) Set(x *{{.ElementName}}) *{{.ElementName}} {
	{{- range $i := .NbWordsIndexesFull}}
		z[{{$i}}] = x[{{$i}}]
	{{- end}}
	return z
}

// SetInterface converts provided interface into {{.ElementName}}
// returns an error if provided type is not supported
// supported types:
//  {{.ElementName}}
//  *{{.ElementName}}
//  uint64
//  int
//  string (see SetString for valid formats)
//  *big.Int
//  big.Int
//  []byte
func (z *{{.ElementName}}) SetInterface(i1 interface{}) (*{{.ElementName}}, error) {
	if i1 == nil {
		return nil, errors.New("can't set {{.PackageName}}.{{.ElementName}} with <nil>")
	}

	switch c1 := i1.(type) {
	case {{.ElementName}}:
		return z.Set(&c1), nil
	case *{{.ElementName}}:
		if c1 == nil {
			return nil, errors.New("can't set {{.PackageName}}.{{.ElementName}} with <nil>")
		}
		return z.Set(c1), nil
	case uint8:
		return z.SetUint64(uint64(c1)), nil
	case uint16:
		return z.SetUint64(uint64(c1)), nil
	case uint32:
		return z.SetUint64(uint64(c1)), nil
	case uint:
		return z.SetUint64(uint64(c1)), nil
	case uint64:
		return z.SetUint64(c1), nil
	case int8:
		return z.SetInt64(int64(c1)), nil
	case int16:
		return z.SetInt64(int64(c1)), nil
	case int32:
		return z.SetInt64(int64(c1)), nil
	case int64:
		return z.SetInt64(c1), nil
	case int:
		return z.SetInt64(int64(c1)), nil
	case string:
		return z.SetString(c1)
	case *big.Int:
		if c1 == nil {
			return nil, errors.New("can't set {{.PackageName}}.{{.ElementName}} with <nil>")
		}
		return z.SetBigInt(c1), nil
	case big.Int:
		return z.SetBigInt(&c1), nil
	case []byte:
		return z.SetBytes(c1), nil
	default:
		return nil, errors.New("can't set {{.PackageName}}.{{.ElementName}} from type " + reflect.TypeOf(i1).String())
	}
}

// SetZero z = 0
func (z *{{.ElementName}}) SetZero() *{{.ElementName}} {
	{{- range $i := .NbWordsIndexesFull}}
		z[{{$i}}] = 0
	{{- end}}
	return z
}

// SetOne z = 1 (in Montgomery form)
func (z *{{.ElementName}}) SetOne() *{{.ElementName}} {
	{{- range $i := .NbWordsIndexesFull}}
		z[{{$i}}] = {{index $.One $i}}
	{{- end}}
	return z
}


// Div z = x*y⁻¹ (mod q)
func (z *{{.ElementName}}) Div( x, y *{{.ElementName}}) *{{.ElementName}} {
	var yInv {{.ElementName}}
	yInv.Inverse( y)
	z.Mul( x, &yInv)
	return z
}

// Equal returns z == x; constant-time
func (z *{{.ElementName}}) Equal(x *{{.ElementName}}) bool {
	return z.NotEqual(x) == 0
}

// NotEqual returns 0 if and only if z == x; constant-time
func (z *{{.ElementName}}) NotEqual(x *{{.ElementName}}) uint64 {
return {{- range $i :=  reverse .NbWordsIndexesNoZero}}(z[{{$i}}] ^ x[{{$i}}]) | {{end}}(z[0] ^ x[0])
}

// IsZero returns z == 0
func (z *{{.ElementName}}) IsZero() bool {
	return ( {{- range $i :=  reverse .NbWordsIndexesNoZero}} z[{{$i}}] | {{end}}z[0]) == 0
}

// IsOne returns z == 1
func (z *{{.ElementName}}) IsOne() bool {
	{{- if eq .NbWords 1}}
	return z[0] == {{index $.One 0}}
	{{- else}}
	return ( {{- range $i := reverse .NbWordsIndexesNoZero -}}{{if ne (index $.One $i) 0}}(z[{{$i}}] ^ {{index $.One $i}}) | {{else}}z[{{$i}}] | {{end}}{{- end}}(z[0] ^ {{index $.One 0}}) ) == 0
	{{- end}}
}

// IsUint64 reports whether z can be represented as an uint64.
func (z *{{.ElementName}}) IsUint64() bool {
	{{- if eq .NbWords 1}}
		return true
	{{- else}}
		zz := *z
		zz.fromMont()
		return zz.FitsOnOneWord()
	{{- end}}
}

// Uint64 returns the uint64 representation of x. If x cannot be represented in a uint64, the result is undefined.
func (z *{{.ElementName}}) Uint64() uint64 {
	return z.Bits()[0]
}

// FitsOnOneWord reports whether z words (except the least significant word) are 0
//
// It is the responsibility of the caller to convert from Montgomery to Regular form if needed.
func (z *{{.ElementName}}) FitsOnOneWord() bool {
	{{- if eq .NbWords 1}}
		return true
	{{- else}}
		return ( {{- range $i :=  reverse .NbWordsIndexesNoZero}} z[{{$i}}] {{- if ne $i 1}}|{{- end}} {{end}}) == 0
	{{- end}}
}

// Cmp compares (lexicographic order) z and x and returns:
//
//   -1 if z <  x
//    0 if z == x
//   +1 if z >  x
//
func (z *{{.ElementName}}) Cmp(x *{{.ElementName}}) int {
	_z := z.Bits()
	_x := x.Bits()
	{{- range $i := reverse $.NbWordsIndexesFull}}
	if _z[{{$i}}] > _x[{{$i}}] {
		return 1
	} else if _z[{{$i}}] < _x[{{$i}}] {
		return -1
	}
	{{- end}}
	return 0
}

// LexicographicallyLargest returns true if this element is strictly lexicographically
// larger than its negation, false otherwise
func (z *{{.ElementName}}) LexicographicallyLargest() bool {
	// adapted from github.com/zkcrypto/bls12_381
	// we check if the element is larger than (q-1) / 2
	// if z - (((q -1) / 2) + 1) have no underflow, then z > (q-1) / 2

	_z := z.Bits()

	var b uint64
	_, b = bits.Sub64(_z[0], {{index .QMinusOneHalvedP 0}}, 0)
	{{- range $i := .NbWordsIndexesNoZero}}
		_, b = bits.Sub64(_z[{{$i}}], {{index $.QMinusOneHalvedP $i}}, b)
	{{- end}}

	return b == 0
}

// SetRandom sets z to a uniform random value in [0, q).
//
// This might error only if reading from crypto/rand.Reader errors,
// in which case, value of z is undefined.
func (z *{{.ElementName}}) SetRandom() (*{{.ElementName}}, error) {
	// this code is generated for all modulus
	// and derived from go/src/crypto/rand/util.go

	// l is number of limbs * 8; the number of bytes needed to reconstruct {{.NbWords}} uint64
	const l = {{mul 8 .NbWords}}

	// bitLen is the maximum bit length needed to encode a value < q.
	const bitLen = {{.NbBits}}

	// k is the maximum byte length needed to encode a value < q.
	const k = (bitLen + 7) / 8

	// b is the number of bits in the most significant byte of q-1.
	b := uint(bitLen % 8)
	if b == 0 {
		b = 8
	}

	var bytes [l]byte

	for {
		// note that bytes[k:l] is always 0
		if _, err := io.ReadFull(rand.Reader, bytes[:k]); err != nil {
			return nil, err
		}

		// Clear unused bits in in the most significant byte to increase probability
		// that the candidate is < q.
		bytes[k-1] &= uint8(int(1<<b) - 1)

		{{- range $i :=  .NbWordsIndexesFull}}
			{{- $k := add $i 1}}
			z[{{$i}}] = binary.LittleEndian.Uint64(bytes[{{mul $i 8}}:{{mul $k 8}}])
		{{- end}}

		if !z.smallerThanModulus() {
			continue // ignore the candidate and re-sample
		}

		return z, nil
	}
}

// smallerThanModulus returns true if z < q
// This is not constant time
func (z *{{.ElementName}}) smallerThanModulus() bool {
	{{- if eq $.NbWords 1}}
		return z[0] < q
	{{- else}}
	return ({{- range $i := reverse .NbWordsIndexesNoZero}} z[{{$i}}] < q{{$i}} || ( z[{{$i}}] == q{{$i}} && (
	{{- end}}z[0] < q0 {{- range $i :=  .NbWordsIndexesNoZero}} )) {{- end}} )
	{{-  end }}
}

// One returns 1
func One() {{.ElementName}} {
	var one {{.ElementName}}
	one.SetOne()
	return one
}

// Halve sets z to z / 2 (mod q)
func (z *{{.ElementName}}) Halve()  {
	{{- if not (and (eq .NbWords 1) (.NoCarry))}}
		var carry uint64
	{{- end}}

	if z[0]&1 == 1 {
		{{- template "add_q" dict "all" . "V1" "z" }}
	}
	{{- rsh "z" .NbWords}}

	{{- if not .NoCarry}}
		if carry != 0 {
			// when we added q, the result was larger than our available limbs
			// when we shift right, we need to set the highest bit
			z[{{.NbWordsLastIndex}}] |= (1 << 63)
		}
	{{end}}
}

{{ define "add_q" }}
	// {{$.V1}} = {{$.V1}} + q
	{{- range $i := $.all.NbWordsIndexesFull }}
		{{- $carryIn := ne $i 0}}
		{{- $carryOut := or (ne $i $.all.NbWordsLastIndex) (and (eq $i $.all.NbWordsLastIndex) (not $.all.NoCarry))}}
		{{$.V1}}[{{$i}}], {{- if $carryOut}}carry{{- else}}_{{- end}} = bits.Add64({{$.V1}}[{{$i}}], q{{$i}}, {{- if $carryIn}}carry{{- else}}0{{- end}})
	{{- end}}
{{ end }}



// fromMont converts z in place (i.e. mutates) from Montgomery to regular representation
// sets and returns z = z * 1
func (z *{{.ElementName}}) fromMont() *{{.ElementName}} {
	fromMont(z)
	return z
}

// Add z = x + y (mod q)
func (z *{{.ElementName}}) Add( x, y *{{.ElementName}}) *{{.ElementName}} {
	{{ $hasCarry := or (not $.NoCarry) (gt $.NbWords 1)}}
	{{- if $hasCarry}}
		var carry uint64
	{{- end}}
	{{- range $i := iterate 0 $.NbWords}}
		{{- $hasCarry := or (not $.NoCarry) (lt $i $.NbWordsLastIndex)}}
		z[{{$i}}], {{- if $hasCarry}}carry{{- else}}_{{- end}} = bits.Add64(x[{{$i}}], y[{{$i}}], {{- if eq $i 0}}0{{- else}}carry{{- end}})
	{{- end}}

	{{- if eq $.NbWords 1}}
		if {{- if not .NoCarry}} carry != 0 ||{{- end }} z[0] >= q {
			z[0] -= q
		}
	{{- else}}
		{{- if not .NoCarry}}
			// if we overflowed the last addition, z >= q
			// if z >= q, z = z - q
			if carry != 0 {
				var b uint64
				// we overflowed, so z >= q
				{{- range $i := iterate 0 $.NbWords}}
					{{- $hasBorrow := lt $i $.NbWordsLastIndex}}
					z[{{$i}}], {{- if $hasBorrow}}b{{- else}}_{{- end}} = bits.Sub64(z[{{$i}}], q{{$i}}, {{- if eq $i 0}}0{{- else}}b{{- end}})
				{{- end}}
				return z
			}
		{{- end}}

		{{ template "reduce" .}}
	{{- end}}
	return z
}

// Double z = x + x (mod q), aka Lsh 1
func (z *{{.ElementName}}) Double( x *{{.ElementName}}) *{{.ElementName}} {
	{{- if eq .NbWords 1}}
	if x[0] & (1 << 63) == (1 << 63) {
		// if highest bit is set, then we have a carry to x + x, we shift and subtract q
		z[0] = (x[0] << 1) - q
	} else {
		// highest bit is not set, but x + x can still be >= q
		z[0] = (x[0] << 1)
		if z[0] >= q {
			z[0] -= q
		}
	}
	{{- else}}
	{{ $hasCarry := or (not $.NoCarry) (gt $.NbWords 1)}}
	{{- if $hasCarry}}
		var carry uint64
	{{- end}}
	{{- range $i := iterate 0 $.NbWords}}
		{{- $hasCarry := or (not $.NoCarry) (lt $i $.NbWordsLastIndex)}}
		z[{{$i}}], {{- if $hasCarry}}carry{{- else}}_{{- end}} = bits.Add64(x[{{$i}}], x[{{$i}}], {{- if eq $i 0}}0{{- else}}carry{{- end}})
	{{- end}}
	{{- if not .NoCarry}}
		// if we overflowed the last addition, z >= q
		// if z >= q, z = z - q
		if carry != 0 {
			var b uint64
			// we overflowed, so z >= q
			{{- range $i := iterate 0 $.NbWords}}
				{{- $hasBorrow := lt $i $.NbWordsLastIndex}}
				z[{{$i}}], {{- if $hasBorrow}}b{{- else}}_{{- end}} = bits.Sub64(z[{{$i}}], q{{$i}}, {{- if eq $i 0}}0{{- else}}b{{- end}})
			{{- end}}
			return z
		}
	{{- end}}

	{{ template "reduce" .}}
	{{- end}}
	return z
}


// Sub z = x - y (mod q)
func (z *{{.ElementName}}) Sub( x, y *{{.ElementName}}) *{{.ElementName}} {
	var b uint64
	z[0], b = bits.Sub64(x[0], y[0], 0)
	{{- range $i := .NbWordsIndexesNoZero}}
		z[{{$i}}], b = bits.Sub64(x[{{$i}}], y[{{$i}}], b)
	{{- end}}
	if b != 0 {
		{{- if eq .NbWords 1}}
			z[0] += q
		{{- else}}
			var c uint64
			z[0], c = bits.Add64(z[0], q0, 0)
			{{- range $i := .NbWordsIndexesNoZero}}
				{{- if eq $i $.NbWordsLastIndex}}
					z[{{$i}}], _ = bits.Add64(z[{{$i}}], q{{$i}}, c)
				{{- else}}
					z[{{$i}}], c = bits.Add64(z[{{$i}}], q{{$i}}, c)
				{{- end}}
			{{- end}}
		{{- end}}
	}
	return z
}

// Neg z = q - x
func (z *{{.ElementName}}) Neg( x *{{.ElementName}}) *{{.ElementName}} {
	if x.IsZero() {
		z.SetZero()
		return z
	}
	{{- if eq .NbWords 1}}
		z[0] = q - x[0]
	{{- else}}
		var borrow uint64
		z[0], borrow = bits.Sub64(q0, x[0], 0)
		{{- range $i := .NbWordsIndexesNoZero}}
			{{- if eq $i $.NbWordsLastIndex}}
				z[{{$i}}], _ = bits.Sub64(q{{$i}}, x[{{$i}}], borrow)
			{{- else}}
				z[{{$i}}], borrow = bits.Sub64(q{{$i}}, x[{{$i}}], borrow)
			{{- end}}
		{{- end}}
	{{- end}}
	return z
}

// Select is a constant-time conditional move.
// If c=0, z = x0. Else z = x1
func (z *{{.ElementName}}) Select(c int, x0 *{{.ElementName}}, x1 *{{.ElementName}}) *{{.ElementName}} {
	cC := uint64( (int64(c) | -int64(c)) >> 63 )	// "canonicized" into: 0 if c=0, -1 otherwise
	{{- range $i := .NbWordsIndexesFull }}
	z[{{$i}}] = x0[{{$i}}] ^ cC & (x0[{{$i}}] ^ x1[{{$i}}])
	{{- end}}
	return z
}

// _mulGeneric is unoptimized textbook CIOS
// it is a fallback solution on x86 when ADX instruction set is not available
// and is used for testing purposes.
func _mulGeneric(z,x,y *{{.ElementName}}) {
	{{ mul_doc false }}
	{{ template "mul_cios" dict "all" . "V1" "x" "V2" "y"}}
	{{ template "reduce"  . }}
}


func _fromMontGeneric(z *{{.ElementName}}) {
	// the following lines implement z = z * 1
	// with a modified CIOS montgomery multiplication
	// see Mul for algorithm documentation
	{{- range $j := .NbWordsIndexesFull}}
	{
		// m = z[0]n'[0] mod W
		m := z[0] * qInvNeg
		C := madd0(m, q0, z[0])
		{{- range $i := $.NbWordsIndexesNoZero}}
			C, z[{{sub $i 1}}] = madd2(m, q{{$i}}, z[{{$i}}], C)
		{{- end}}
		z[{{sub $.NbWords 1}}] = C
	}
	{{- end}}

	{{ template "reduce" .}}
}

func _reduceGeneric(z *{{.ElementName}})  {
	{{ template "reduce"  . }}
}

// BatchInvert returns a new slice with every element inverted.
// Uses Montgomery batch inversion trick
func BatchInvert(a []{{.ElementName}}) []{{.ElementName}} {
	res := make([]{{.ElementName}}, len(a))
	if len(a) == 0 {
		return res
	}

	zeroes := bitset.New(uint(len(a)))
	accumulator := One()

	for i:=0; i < len(a); i++ {
		if a[i].IsZero() {
			zeroes.Set(uint(i))
			continue
		}
		res[i] = accumulator
		accumulator.Mul(&accumulator, &a[i])
	}

	accumulator.Inverse(&accumulator)

	for i := len(a) - 1; i >= 0; i-- {
		if zeroes.Test(uint(i)) {
			continue
		}
		res[i].Mul(&res[i], &accumulator)
		accumulator.Mul(&accumulator, &a[i])
	}

	return res
}

func _butterflyGeneric(a, b *{{.ElementName}}) {
	t := *a
	a.Add(a, b)
	b.Sub(&t, b)
}

// BitLen returns the minimum number of bits needed to represent z
// returns 0 if z == 0
func (z *{{.ElementName}}) BitLen() int {
	{{- range $i := reverse .NbWordsIndexesNoZero}}
	if z[{{$i}}] != 0 {
		return {{mul $i 64}} + bits.Len64(z[{{$i}}])
	}
	{{- end}}
	return bits.Len64(z[0])
}

// Hash msg to count prime field elements.
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-5.2
func Hash(msg, dst []byte, count int) ([]{{.ElementName}}, error) {
	// 128 bits of security
	// L = ceil((ceil(log2(p)) + k) / 8), where k is the security parameter = 128
	const Bytes = 1 + (Bits-1)/8
	const L = 16 + Bytes

	lenInBytes := count * L
	pseudoRandomBytes, err := hash.ExpandMsgXmd(msg, dst, lenInBytes)
	if err != nil {
		return nil, err
	}

	// get temporary big int from the pool
	vv := pool.BigInt.Get()

	res := make([]{{.ElementName}}, count)
	for i := 0; i < count; i++ {
		vv.SetBytes(pseudoRandomBytes[i*L : (i+1)*L])
		res[i].SetBigInt(vv)
	}

	// release object into pool
	pool.BigInt.Put(vv)

	return res, nil
}


{{ define "rsh V nbWords" }}
	// {{$.V}} = {{$.V}} >> 1
	{{- $lastIndex := sub .nbWords 1}}
	{{- range $i :=  iterate 0 $lastIndex}}
		{{$.V}}[{{$i}}] = {{$.V}}[{{$i}}] >> 1 | {{$.V}}[{{(add $i 1)}}] << 63
	{{- end}}
	{{$.V}}[{{$lastIndex}}] >>= 1
{{ end }}



`
