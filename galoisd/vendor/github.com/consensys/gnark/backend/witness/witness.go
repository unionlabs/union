// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package witness provides serialization helpers to encode a witness into a []byte.
//
// Binary protocol
//
//	Witness     ->  [uint32(nbPublic) | uint32(nbSecret) | fr.Vector(variables)]
//	fr.Vector is a *field element* vector encoded a big-endian byte array like so: [uint32(len(vector)) | elements]
//
// # Ordering
//
// First, `publicVariables`, then `secretVariables`. Each subset is ordered from the order of definition in the circuit structure.
// For example, with this circuit on `ecc.BN254`
//
//	type Circuit struct {
//	    X frontend.Variable
//	    Y frontend.Variable `gnark:",public"`
//	    Z frontend.Variable
//	}
//
// A valid witness would be:
//   - `[uint32(1)|uint32(2)|uint32(3)|bytes(Y)|bytes(X)|bytes(Z)]`
//   - Hex representation with values `Y = 35`, `X = 3`, `Z = 2`
//     `000000010000000200000003000000000000000000000000000000000000000000000000000000000000002300000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000002`
package witness

import (
	"bytes"
	"encoding"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math/big"
	"reflect"

	fr_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr"
	fr_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	fr_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr"
	fr_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	fr_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr"
	fr_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/internal/tinyfield"
)

var ErrInvalidWitness = errors.New("invalid witness")

// Witness represents a zkSNARK witness.
//
// The underlying data structure is a vector of field elements, but a Witness
// also may have some additional meta information about the number of public elements and
// secret elements.
//
// In most cases a Witness should be [de]serialized using a binary protocol.
// JSON conversions for pretty printing are slow and don't handle all complex circuit structures well.
type Witness interface {
	io.WriterTo
	io.ReaderFrom
	encoding.BinaryMarshaler
	encoding.BinaryUnmarshaler

	// Public returns the Public an object containing the public part of the Witness only.
	Public() (Witness, error)

	// Vector returns the underlying fr.Vector slice
	Vector() any

	// ToJSON returns the JSON encoding of the witness following the provided Schema. This is a
	// convenience method and should be avoided in most cases.
	ToJSON(s *schema.Schema) ([]byte, error)

	// FromJSON parses a JSON data input and attempt to reconstruct a witness following the provided Schema.
	// This is a convenience method and should be avoided in most cases.
	FromJSON(s *schema.Schema, data []byte) error

	// Fill range over the provided chan to fill the underlying vector.
	// Will allocate the underlying vector with nbPublic + nbSecret elements.
	// This is typically call by internal APIs to fill the vector by walking a structure.
	Fill(nbPublic, nbSecret int, values <-chan any) error
}

type witness struct {
	vector             any
	nbPublic, nbSecret uint32
}

// New initialize a new empty Witness.
func New(field *big.Int) (Witness, error) {
	v, err := newVector(field, 0)
	if err != nil {
		return nil, err
	}

	return &witness{
		vector: v,
	}, nil
}

func (w *witness) Fill(nbPublic, nbSecret int, values <-chan any) error {
	n := nbPublic + nbSecret
	w.vector = resize(w.vector, n)
	w.nbPublic = uint32(nbPublic)
	w.nbSecret = uint32(nbSecret)

	i := 0

	// note; this shouldn't be perf critical but if it is we could have 2 input chan and
	// fill public and secret values concurrently.
	for v := range values {
		if i >= n {
			// we panic here; shouldn't happen and if it does we may leek a chan + producer go routine
			panic("chan of values returns more elements than expected")
		}
		// if v == nil {
		// 	this is caught in the set method. however, error message will be unclear; reason
		// is there is a nil field in assignment, we could print which one.
		// }
		if err := set(w.vector, i, v); err != nil {
			return err
		}
		i++
	}

	if i != n {
		return fmt.Errorf("expected %d values, filled only %d", n, i)
	}

	return nil
}

func (w *witness) iterate() chan any {
	return iterate(w.vector)
}

func (w *witness) Public() (Witness, error) {
	v, err := newFrom(w.vector, int(w.nbPublic))
	if err != nil {
		return nil, err
	}
	return &witness{
		vector:   v,
		nbPublic: w.nbPublic,
	}, nil
}

func (w *witness) WriteTo(wr io.Writer) (n int64, err error) {
	// write number of public, number of secret
	if err := binary.Write(wr, binary.BigEndian, w.nbPublic); err != nil {
		return 0, err
	}
	n = int64(4)
	if err := binary.Write(wr, binary.BigEndian, w.nbSecret); err != nil {
		return n, err
	}
	n += 4

	// write the vector
	var m int64
	switch t := w.vector.(type) {
	case fr_bn254.Vector:
		m, err = t.WriteTo(wr)
	case fr_bls12377.Vector:
		m, err = t.WriteTo(wr)
	case fr_bls12381.Vector:
		m, err = t.WriteTo(wr)
	case fr_bw6761.Vector:
		m, err = t.WriteTo(wr)
	case fr_bls24317.Vector:
		m, err = t.WriteTo(wr)
	case fr_bls24315.Vector:
		m, err = t.WriteTo(wr)
	case fr_bw6633.Vector:
		m, err = t.WriteTo(wr)
	case tinyfield.Vector:
		m, err = t.WriteTo(wr)
	default:
		panic("invalid input")
	}
	n += m
	return n, err
}

func (w *witness) ReadFrom(r io.Reader) (n int64, err error) {
	var buf [4]byte
	if read, err := io.ReadFull(r, buf[:]); err != nil {
		return int64(read), err
	}
	w.nbPublic = binary.BigEndian.Uint32(buf[:4])
	if read, err := io.ReadFull(r, buf[:]); err != nil {
		return int64(read) + 4, err
	}
	w.nbSecret = binary.BigEndian.Uint32(buf[:4])

	n = 8

	var m int64
	switch t := w.vector.(type) {
	case fr_bn254.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bls12377.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bls12381.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bw6761.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bls24317.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bls24315.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case fr_bw6633.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	case tinyfield.Vector:
		m, err = t.ReadFrom(r)
		w.vector = t
	default:
		panic("invalid input")
	}

	n += m
	return n, err
}

// MarshalBinary encodes the number of public, number of secret and the fr.Vector.
func (w *witness) MarshalBinary() (data []byte, err error) {
	var buf bytes.Buffer

	if _, err = w.WriteTo(&buf); err != nil {
		return
	}
	return buf.Bytes(), nil
}

// UnmarshalBinary implements encoding.BinaryUnmarshaler
func (w *witness) UnmarshalBinary(data []byte) error {
	r := bytes.NewReader(data)
	_, err := w.ReadFrom(r)
	return err
}

func (w *witness) Vector() any {
	return w.vector
}

// ToJSON returns the JSON encoding of the witness following the provided Schema. This is a
// convenience method and should be avoided in most cases.
func (w *witness) ToJSON(s *schema.Schema) ([]byte, error) {
	if s.NbPublic != int(w.nbPublic) || (w.nbSecret != 0 && w.nbSecret != uint32(s.NbSecret)) {
		return nil, errors.New("schema is inconsistent with Witness")
	}
	typ := reflect.PtrTo(leafType(w.vector))
	instance := s.Instantiate(typ)

	chValues := w.iterate()
	if _, err := schema.Walk(instance, typ, func(field schema.LeafInfo, tValue reflect.Value) error {
		if field.Visibility == schema.Public {
			v := <-chValues
			tValue.Set(reflect.ValueOf(v))
		}
		return nil
	}); err != nil {
		return nil, err
	}

	if w.nbSecret != 0 {
		// secret part.
		if _, err := schema.Walk(instance, typ, func(field schema.LeafInfo, tValue reflect.Value) error {
			if field.Visibility == schema.Secret {
				v := <-chValues
				tValue.Set(reflect.ValueOf(v))
			}
			return nil
		}); err != nil {
			return nil, err
		}
	}

	if debug.Debug {
		return json.MarshalIndent(instance, "  ", "    ")
	} else {
		return json.Marshal(instance)
	}
}

// FromJSON parses a JSON data input and attempt to reconstruct a witness following the provided Schema.
// This is a convenience method and should be avoided in most cases.
func (w *witness) FromJSON(s *schema.Schema, data []byte) error {
	typ := leafType(w.vector)
	ptrTyp := reflect.PtrTo(typ)

	// we instantiate an object matching the schema, with leaf type == field element
	// note that we pass a pointer here to have nil for zero values
	instance := s.Instantiate(ptrTyp)

	dec := json.NewDecoder(bytes.NewReader(data))
	dec.DisallowUnknownFields()

	// field.Element (gnark-crypto) implements json.Unmarshaler
	if err := dec.Decode(instance); err != nil {
		return err
	}
	// walk through the public AND secret values
	missingAssignment := func(name string) error {
		return fmt.Errorf("missing assignment for %s", name)
	}

	// collect all public values; if any are missing, no point going further.
	publicValues := make([]any, 0, s.NbPublic)
	if _, err := schema.Walk(instance, ptrTyp, func(leaf schema.LeafInfo, tValue reflect.Value) error {
		if leaf.Visibility == schema.Public {
			if tValue.IsNil() {
				return missingAssignment(leaf.FullName())
			}
			publicValues = append(publicValues, reflect.Indirect(tValue).Interface())
		}
		return nil
	}); err != nil {
		// missing public values
		return err
	}

	// collect all secret values; if any are missing, we just deal with the public part.
	secretValues := make([]any, 0, s.NbSecret)
	publicOnly := false
	if _, err := schema.Walk(instance, ptrTyp, func(leaf schema.LeafInfo, tValue reflect.Value) error {
		if leaf.Visibility == schema.Secret {
			if tValue.IsNil() {
				return missingAssignment(leaf.FullName())
			}
			secretValues = append(secretValues, reflect.Indirect(tValue).Interface())
		}
		return nil
	}); err != nil {
		// missing secret values, we just do the public part.
		publicOnly = true
	}

	// reconstruct the witness
	// we use a buffered channel to ensure this go routine terminates, even if setting a witness
	// value failed. All this is not really performant for large witnesses, but again, JSON
	// shouldn't be used in perf-critical scenario.
	var chValues chan any
	if publicOnly {
		chValues = make(chan any, len(publicValues))
		s.NbSecret = 0
	} else {
		chValues = make(chan any, len(publicValues)+len(secretValues))
	}
	go func() {
		defer close(chValues)

		for _, v := range publicValues {
			chValues <- v
		}

		if publicOnly {
			return
		}

		for _, v := range secretValues {
			chValues <- v
		}
	}()

	return w.Fill(s.NbPublic, s.NbSecret, chValues)
}
