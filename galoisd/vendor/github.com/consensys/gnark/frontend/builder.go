package frontend

import (
	"math/big"

	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend/schema"
)

type NewBuilder func(*big.Int, CompileConfig) (Builder, error)

// Compiler represents a constraint system compiler
type Compiler interface {
	constraint.CustomizableSystem

	// MarkBoolean sets (but do not constraint!) v to be boolean
	// This is useful in scenarios where a variable is known to be boolean through a constraint
	// that is not api.AssertIsBoolean. If v is a constant, this is a no-op.
	MarkBoolean(v Variable)

	// IsBoolean returns true if given variable was marked as boolean in the compiler (see MarkBoolean)
	// Use with care; variable may not have been **constrained** to be boolean
	// This returns true if the v is a constant and v == 0 || v == 1.
	IsBoolean(v Variable) bool

	// NewHint initializes internal variables whose value will be evaluated
	// using the provided hint function at run time from the inputs. Inputs must
	// be either variables or convertible to *big.Int. The function returns an
	// error if the number of inputs is not compatible with f.
	//
	// The hint function is provided at the proof creation time and is not
	// embedded into the circuit. From the backend point of view, the variable
	// returned by the hint function is equivalent to the user-supplied witness,
	// but its actual value is assigned by the solver, not the caller.
	//
	// No new constraints are added to the newly created wire and must be added
	// manually in the circuit. Failing to do so leads to solver failure.
	//
	// If nbOutputs is specified, it must be >= 1 and <= f.NbOutputs
	NewHint(f solver.Hint, nbOutputs int, inputs ...Variable) ([]Variable, error)

	// ConstantValue returns the big.Int value of v and true if op is a success.
	// nil and false if failure. This API returns a boolean to allow for future refactoring
	// replacing *big.Int with fr.Element
	ConstantValue(v Variable) (*big.Int, bool)

	// Field returns the finite field modulus injected by the compiler
	Field() *big.Int

	// FieldBitLen returns the number of bits needed to represent an element in the scalar field
	FieldBitLen() int

	// Defer is called after circuit.Define() and before Compile(). This method
	// allows for the circuits to register callbacks which finalize batching
	// operations etc. Unlike Go defer, it is not locally scoped.
	Defer(cb func(api API) error)

	// InternalVariable returns the internal variable associated with the given wireID
	// ! Experimental: use in conjunction with constraint.CustomizableSystem
	InternalVariable(wireID uint32) Variable

	// ToCanonicalVariable converts a frontend.Variable to a constraint system specific Variable
	// ! Experimental: use in conjunction with constraint.CustomizableSystem
	ToCanonicalVariable(Variable) CanonicalVariable

	SetGkrInfo(constraint.GkrInfo) error
}

// Builder represents a constraint system builder
type Builder interface {
	API
	Compiler

	// Compile is called after circuit.Define() to produce a final IR (ConstraintSystem)
	Compile() (constraint.ConstraintSystem, error)

	// PublicVariable is called by the compiler when parsing the circuit schema. It panics if
	// called inside circuit.Define()
	PublicVariable(schema.LeafInfo) Variable

	// SecretVariable is called by the compiler when parsing the circuit schema. It panics if
	// called inside circuit.Define()
	SecretVariable(schema.LeafInfo) Variable
}

// Committer allows to commit to the variables and returns the commitment. The
// commitment can be used as a challenge using Fiat-Shamir heuristic.
type Committer interface {
	// Commit commits to the variables and returns the commitment.
	Commit(toCommit ...Variable) (commitment Variable, err error)
}

// Rangechecker allows to externally range-check the variables to be of
// specified width. Not all compilers implement this interface. Users should
// instead use [github.com/consensys/gnark/std/rangecheck] package which
// automatically chooses most optimal method for range checking the variables.
type Rangechecker interface {
	// Check checks that the given variable v has bit-length bits.
	Check(v Variable, bits int)
}

// CanonicalVariable represents a variable that's encoded in a constraint system specific way.
// For example a R1CS builder may represent this as a constraint.LinearExpression,
// a PLONK builder --> constraint.Term
// and the test/Engine --> ~*big.Int.
type CanonicalVariable interface {
	constraint.Compressible
}
