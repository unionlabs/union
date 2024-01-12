/*
Copyright © 2020 ConsenSys

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Package mimc provides a ZKP-circuit function to compute a MiMC hash.
package mimc

import (
	"errors"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/utils"
)

// MiMC contains the params of the Mimc hash func and the curves on which it is implemented
type MiMC struct {
	params []big.Int           // slice containing constants for the encryption rounds
	id     ecc.ID              // id needed to know which encryption function to use
	h      frontend.Variable   // current vector in the Miyaguchi–Preneel scheme
	data   []frontend.Variable // state storage. data is updated when Write() is called. Sum sums the data.
	api    frontend.API        // underlying constraint system
}

// NewMiMC returns a MiMC instance, that can be used in a gnark circuit
func NewMiMC(api frontend.API) (MiMC, error) {
	// TODO @gbotrel use field
	if constructor, ok := newMimc[utils.FieldToCurve(api.Compiler().Field())]; ok {
		return constructor(api), nil
	}
	return MiMC{}, errors.New("unknown curve id")
}

// Write adds more data to the running hash.
func (h *MiMC) Write(data ...frontend.Variable) {
	h.data = append(h.data, data...)
}

// Reset resets the Hash to its initial state.
func (h *MiMC) Reset() {
	h.data = nil
	h.h = 0
}

// Sum hash (in r1cs form) using Miyaguchi–Preneel:
// https://en.wikipedia.org/wiki/One-way_compression_function
// The XOR operation is replaced by field addition.
// See github.com/consensys/gnark-crypto for reference implementation.
func (h *MiMC) Sum() frontend.Variable {

	//h.Write(data...)s
	for _, stream := range h.data {
		r := encryptFuncs[h.id](*h, stream)
		h.h = h.api.Add(h.h, r, stream)
	}

	h.data = nil // flush the data already hashed

	return h.h

}
