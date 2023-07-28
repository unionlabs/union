package frontend

import (
	"math/big"
	"reflect"

	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend/schema"
)

// NewWitness build an ordered vector of field elements from the given assignment (Circuit)
// if PublicOnly is specified, returns the public part of the witness only
// else returns [public | secret]. The result can then be serialized to / from json & binary.
//
// See ExampleWitness in witness package for usage.
func NewWitness(assignment Circuit, field *big.Int, opts ...WitnessOption) (witness.Witness, error) {
	opt, err := options(opts...)
	if err != nil {
		return nil, err
	}

	// count the leaves
	s, err := schema.Walk(assignment, tVariable, nil)
	if err != nil {
		return nil, err
	}
	if opt.publicOnly {
		s.Secret = 0
	}

	// allocate the witness
	w, err := witness.New(field)
	if err != nil {
		return nil, err
	}

	// write the public | secret values in a chan
	chValues := make(chan any)
	go func() {
		defer close(chValues)
		schema.Walk(assignment, tVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
			if leaf.Visibility == schema.Public {
				chValues <- tValue.Interface()
			}
			return nil
		})
		if !opt.publicOnly {
			schema.Walk(assignment, tVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
				if leaf.Visibility == schema.Secret {
					chValues <- tValue.Interface()
				}
				return nil
			})
		}
	}()
	if err := w.Fill(s.Public, s.Secret, chValues); err != nil {
		return nil, err
	}

	return w, nil
}

// NewSchema returns the schema corresponding to the circuit structure.
//
// This is used to JSON (un)marshall witnesses.
func NewSchema(circuit Circuit) (*schema.Schema, error) {
	return schema.New(circuit, tVariable)
}

// default options
func options(opts ...WitnessOption) (witnessConfig, error) {
	// apply options
	opt := witnessConfig{
		publicOnly: false,
	}
	for _, option := range opts {
		if err := option(&opt); err != nil {
			return opt, err
		}
	}

	return opt, nil
}

// WitnessOption sets optional parameter to witness instantiation from an assignment
type WitnessOption func(*witnessConfig) error

type witnessConfig struct {
	publicOnly bool
}

// PublicOnly enables to instantiate a witness with the public part only of the assignment
func PublicOnly() WitnessOption {
	return func(opt *witnessConfig) error {
		opt.publicOnly = true
		return nil
	}
}
