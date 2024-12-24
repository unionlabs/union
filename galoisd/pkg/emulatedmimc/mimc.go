package emulatedmimc

import (
	"errors"
	"math/big"

	// bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr/mimc"
	// bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr/mimc"
	// bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr/mimc"
	bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	// bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr/mimc"
	// "github.com/consensys/gnark/std/algebra/emulated/sw_bls12381"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"

	// "github.com/consensys/gnark/std/algebra/emulated/sw_bw6761"
	// "github.com/consensys/gnark/std/algebra/native/sw_bls12377"
	// "github.com/consensys/gnark/std/algebra/native/sw_bls24315"
	"github.com/consensys/gnark/std/math/emulated"
)

// MiMC contains the params of the Mimc hash func and the curves on which it is implemented
type MiMC[T emulated.FieldParams] struct {
	encrypt func(MiMC[T], *emulated.Element[T]) *emulated.Element[T]
	params  []*emulated.Element[T] // slice containing constants for the encryption rounds
	h       *emulated.Element[T]   // current vector in the Miyaguchi–Preneel scheme
	data    []*emulated.Element[T] // state storage. data is updated when Write() is called. Sum sums the data.
	field   *emulated.Field[T]
}

// NewMiMC returns a MiMC instance, that can be used in a gnark circuit
func NewMiMC[T emulated.FieldParams](field *emulated.Field[T]) (MiMC[T], error) {
	var encrypt func(MiMC[T], *emulated.Element[T]) *emulated.Element[T]
	var params []big.Int
	switch any(field).(type) {
	case *emulated.Field[sw_bn254.ScalarField]:
		encrypt = encryptPow5[T]
		params = bn254.GetConstants()
	// case *emulated.Field[sw_bls12381.ScalarField]:
	// 	encrypt = encryptPow5[T]
	// 	params = bls12381.GetConstants()
	// case *emulated.Field[sw_bls12377.ScalarField]:
	// 	encrypt = encryptPow17[T]
	// 	params = bls12377.GetConstants()
	// case *emulated.Field[sw_bw6761.ScalarField]:
	// 	encrypt = encryptPow5[T]
	// 	params = bw6761.GetConstants()
	// case *emulated.Field[sw_bls24315.ScalarField]:
	// 	encrypt = encryptPow5[T]
	// 	params = bls24315.GetConstants()
	default:
		return MiMC[T]{}, errors.New("unknown curve id")
	}
	paramsE := make([]*emulated.Element[T], len(params))
	for i, p := range params {
		e := emulated.ValueOf[T](p)
		paramsE[i] = &e
	}
	return MiMC[T]{
		encrypt: encrypt,
		params:  paramsE,
		h:       field.Zero(),
		field:   field,
	}, nil
}

// Write adds more data to the running hash.
func (h *MiMC[T]) Write(data ...*emulated.Element[T]) {
	h.data = append(h.data, data...)
}

// Reset resets the Hash to its initial state.
func (h *MiMC[T]) Reset() {
	h.data = nil
	h.h = h.field.Zero()
}

// Sum hash (in r1cs form) using Miyaguchi–Preneel:
// https://en.wikipedia.org/wiki/One-way_compression_function
// The XOR operation is replaced by field addition.
// See github.com/consensys/gnark-crypto for reference implementation.
func (h *MiMC[T]) Sum() *emulated.Element[T] {
	for _, stream := range h.data {
		r := h.encrypt(*h, stream)
		h.h = h.field.Add(h.field.Add(h.h, r), stream)
	}
	h.data = nil // flush the data already hashed
	return h.h
}
