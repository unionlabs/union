package frontend

import (
	"errors"
	"fmt"
	"math/big"
	"reflect"

	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/internal/circuitdefer"
	"github.com/consensys/gnark/logger"
)

// Compile will generate a ConstraintSystem from the given circuit
//
// 1. it will first allocate the user inputs (see [schema.TagOpt] and [Circuit] for more info)
// example:
//
//	type MyCircuit struct {
//		Y frontend.Variable `gnark:"exponent,public"`
//	}
//
// in that case, Compile() will allocate one public variable with id "exponent"
//
// 2. it then calls circuit.Define(curveID, R1CS) to build the internal constraint system
// from the declarative code
//
//  3. finally, it converts that to a ConstraintSystem.
//     if zkpID == backend.GROTH16	→ R1CS
//     if zkpID == backend.PLONK 	→ SparseR1CS
//
// initialCapacity is an optional parameter that reserves memory in slices
// it should be set to the estimated number of constraints in the circuit, if known.
func Compile(field *big.Int, newBuilder NewBuilder, circuit Circuit, opts ...CompileOption) (constraint.ConstraintSystem, error) {
	log := logger.Logger()
	log.Info().Msg("compiling circuit")
	// parse options
	opt := defaultCompileConfig()
	for _, o := range opts {
		if err := o(&opt); err != nil {
			log.Err(err).Msg("applying compile option")
			return nil, fmt.Errorf("apply option: %w", err)
		}
	}

	// instantiate new builder
	builder, err := newBuilder(field, opt)
	if err != nil {
		log.Err(err).Msg("instantiating builder")
		return nil, fmt.Errorf("new compiler: %w", err)
	}

	// parse the circuit builds a schema of the circuit
	// and call circuit.Define() method to initialize a list of constraints in the compiler
	if err = parseCircuit(builder, circuit); err != nil {
		log.Err(err).Msg("parsing circuit")
		return nil, fmt.Errorf("parse circuit: %w", err)

	}

	// compile the circuit into its final form
	return builder.Compile()
}

func parseCircuit(builder Builder, circuit Circuit) (err error) {
	// ensure circuit.Define has pointer receiver
	if reflect.ValueOf(circuit).Kind() != reflect.Ptr {
		return errors.New("frontend.Circuit methods must be defined on pointer receiver")
	}

	s, err := schema.Walk(circuit, tVariable, nil)
	if err != nil {
		return err
	}

	log := logger.Logger()
	log.Info().Int("nbSecret", s.Secret).Int("nbPublic", s.Public).Msg("parsed circuit inputs")

	// leaf handlers are called when encoutering leafs in the circuit data struct
	// leafs are Constraints that need to be initialized in the context of compiling a circuit
	variableAdder := func(targetVisibility schema.Visibility) func(f schema.LeafInfo, tInput reflect.Value) error {
		return func(f schema.LeafInfo, tInput reflect.Value) error {
			if tInput.CanSet() {
				if f.Visibility == schema.Unset {
					return errors.New("can't set val " + f.FullName() + " visibility is unset")
				}
				if f.Visibility == targetVisibility {
					if f.Visibility == schema.Public {
						tInput.Set(reflect.ValueOf(builder.PublicVariable(f)))
					} else if f.Visibility == schema.Secret {
						tInput.Set(reflect.ValueOf(builder.SecretVariable(f)))
					}
				}

				return nil
			}
			return errors.New("can't set val " + f.FullName())
		}
	}

	// add public inputs first to compute correct offsets
	_, err = schema.Walk(circuit, tVariable, variableAdder(schema.Public))
	if err != nil {
		return err
	}

	// add secret inputs
	_, err = schema.Walk(circuit, tVariable, variableAdder(schema.Secret))
	if err != nil {
		return err
	}

	// recover from panics to print user-friendlier messages
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("%v\n%s", r, debug.Stack())
		}
	}()

	// call Define() to fill in the Constraints
	if err = circuit.Define(builder); err != nil {
		return fmt.Errorf("define circuit: %w", err)
	}
	if err = callDeferred(builder); err != nil {
		return fmt.Errorf("deferred: %w", err)
	}

	return
}

func callDeferred(builder Builder) error {
	for i := 0; i < len(circuitdefer.GetAll[func(API) error](builder)); i++ {
		if err := circuitdefer.GetAll[func(API) error](builder)[i](builder); err != nil {
			return fmt.Errorf("defer fn %d: %w", i, err)
		}
	}
	return nil
}

// CompileOption defines option for altering the behaviour of the Compile
// method. See the descriptions of the functions returning instances of this
// type for available options.
type CompileOption func(opt *CompileConfig) error

func defaultCompileConfig() CompileConfig {
	return CompileConfig{
		CompressThreshold: 300,
	}
}

type CompileConfig struct {
	Capacity                  int
	IgnoreUnconstrainedInputs bool
	CompressThreshold         int
}

// WithCapacity is a compile option that specifies the estimated capacity needed
// for internal variables and constraints. If not set, then the initial capacity
// is 0 and is dynamically allocated as needed.
func WithCapacity(capacity int) CompileOption {
	return func(opt *CompileConfig) error {
		opt.Capacity = capacity
		return nil
	}
}

// IgnoreUnconstrainedInputs is a compile option which allow compiling input
// circuits where not all inputs are not constrained. If not set, then the
// compiler returns an error if there exists an unconstrained input.
//
// This option is useful for debugging circuits, but should not be used in
// production settings as it means that there is a potential error in the
// circuit definition or that it is possible to optimize witness size.
func IgnoreUnconstrainedInputs() CompileOption {
	return func(opt *CompileConfig) error {
		opt.IgnoreUnconstrainedInputs = true
		return nil
	}
}

// WithCompressThreshold is a compile option which enforces automatic variable
// compression if the length of the linear expression in the variable exceeds
// given threshold.
//
// This option is usable in arithmetisations where the variable is a linear
// combination, as for example in R1CS. If variable is not a linear combination,
// then this option does not change the compile behaviour.
//
// This compile option should be used in cases when it is known that there are
// long addition chains and the compile time and memory usage start are growing
// fast. The compression adds some overhead in the number of constraints. The
// overhead and compile performance depends on threshold value, and it should be
// chosen carefully.
//
// If this option is not given then by default we use the compress threshold of
// 300.
func WithCompressThreshold(threshold int) CompileOption {
	return func(opt *CompileConfig) error {
		opt.CompressThreshold = threshold
		return nil
	}
}

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A Variable }{}).FieldByName("A").Type()
}
