package solver

import (
	"hash/fnv"
	"math/big"
	"reflect"
	"regexp"
	"runtime"
)

// HintID is a unique identifier for a hint function used for lookup.
type HintID uint32

// Hint allows to define computations outside of a circuit.
//
// It defines an annotated hint function; the number of inputs and outputs injected at solving
// time is defined in the circuit (compile time).
//
// The slices inputs and outputs are already initialized and the also the
// elements within. The elements come from the pool, so leaking the values leads
// to undefined behaviour. As elements in output slice are already initialized,
// then use [math/big.Int.Set] method to assign value.
//
// The value field defines the current field of definition. Usually the circuits
// are typed to particular fields, but this allows to use a single hint function
// for all different curves.
//
// When the hint function returns an error, then this leads to solver error during the proving stage.
//
// For example:
//
//	b := api.NewHint(hint, 2, a)
//	--> at solving time, hint is going to be invoked with 1 input (a) and is expected to return 2 outputs
//	b[0] and b[1].
//
// Usually, it is expected that computations in circuits are performed on
// variables. However, in some cases defining the computations in circuits may be
// complicated or computationally expensive. By using hints, the computations are
// performed outside of the circuit on integers (compared to the frontend.Variable
// values inside the circuits) and the result of a hint function is assigned to a
// newly created variable in a circuit.
//
// As the computations are performed outside of the circuit, then the correctness of
// the result is not guaranteed. This also means that the result of a hint function
// is unconstrained by default, leading to failure while composing circuit proof.
// Thus, it is the circuit developer responsibility to verify the correctness hint
// result by adding necessary constraints in the circuit.
//
// As an example, lets say the hint function computes a factorization of a
// semiprime n:
//
//	p, q <- hint(n) st. p * q = n
//
// into primes p and q. Then, the circuit developer needs to assert in the circuit
// that p*q indeed equals to n:
//
//	n == p * q.
//
// However, if the hint function is incorrectly defined (e.g. in the previous
// example, it returns 1 and n instead of p and q), then the assertion may still
// hold, but the constructed proof is semantically invalid. Thus, the user
// constructing the proof must be extremely cautious when using hints.
//
// # Using hint functions in circuits
//
// To use a hint function in a circuit, the developer first needs to define a hint
// function hintFn according to the Function interface. Then, in a circuit, the
// developer applies the hint function with frontend.API.NewHint(hintFn, vars...),
// where vars are the variables the hint function will be applied to (and
// correspond to the argument inputs in the Function type) which returns a new
// unconstrained variable. The returned variables must be constrained using
// frontend.API.Assert[.*] methods.
//
// As explained, the hints are essentially black boxes from the circuit point of
// view and thus the defined hints in circuits are not used when constructing a
// proof. To allow the particular hint functions to be used during proof
// construction, the user needs to supply a solver.Option indicating the
// enabled hints. Such options can be obtained by a call to
// [solver.WithHints], where hintFns are the corresponding hint
// functions.
//
// # Using hint functions in gadgets
//
// Similar considerations apply for hint functions used in gadgets as in
// user-defined circuits. However, listing all hint functions used in a particular
// gadget for constructing solver.Option puts high overhead for the user to
// enable all necessary hints.
//
// For that, this package also provides a registry of trusted hint functions. When
// a gadget registers a hint function, then it is automatically enabled during
// proof computation and the prover does not need to provide a corresponding
// proving option.
//
// In the init() method of the gadget, call the method [RegisterHint] function on
// the hint function hintFn to register a hint function in the package registry.
type Hint func(field *big.Int, inputs []*big.Int, outputs []*big.Int) error

// GetHintID returns the derived hint ID from the hint function reference.
func GetHintID(fn Hint) HintID {
	hf := fnv.New32a()
	name := GetHintName(fn)

	// TODO relying on name to derive UUID is risky; if fn is an anonymous func, wil be package.glob..funcN
	// and if new anonymous functions are added in the package, N may change, so will UUID.
	hf.Write([]byte(name)) // #nosec G104 -- does not err

	return HintID(hf.Sum32())
}

// GetHintName returns the derived hint name from the hint function reference.
// By default, it is the fully qualified name of the function. If the function
// is anonymous, then it is the fully qualified name of the package and the
// function index.
func GetHintName(fn Hint) string {
	fnptr := reflect.ValueOf(fn).Pointer()
	name := runtime.FuncForPC(fnptr).Name()
	return newToOldStyle(name)
}

func newToOldStyle(name string) string {
	return string(newStyleAnonRe.ReplaceAll([]byte(name), []byte("${pkgname}glob.${funcname}")))
}

var newStyleAnonRe = regexp.MustCompile(`^(?P<pkgname>.*\.)init(?P<funcname>\.func\d+)$`)
