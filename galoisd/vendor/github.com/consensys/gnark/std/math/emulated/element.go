package emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/utils"
)

// Element defines an element in the ring of integers modulo n. The integer
// value of the element is split into limbs of nbBits lengths and represented as
// a slice of limbs. The type parameter defines the field this element belongs
// to.
type Element[T FieldParams] struct {
	// Limbs is the decomposition of the integer value into limbs in the native
	// field. To enforce that the limbs are of expected width, use Pack...
	// methods on the Field. Uses little-endian (least significant limb first)
	// encoding.
	Limbs []frontend.Variable

	// overflow indicates the number of additions on top of the normal form. To
	// ensure that none of the limbs overflow the scalar field of the snark
	// curve, we must check that nbBits+overflow < floor(log2(fr modulus))
	overflow uint

	// internal indicates if the element is returned from [Field] methods. If
	// so, then we can assume that the limbs are already constrained to be
	// correct width. If the flag is not set, then the Element most probably
	// comes from the witness (or constructed by the user). Then we have to
	// ensure that the limbs are width-constrained. We do not store the
	// enforcement info in the Element to prevent modifying the witness.
	internal bool

	isEvaluated bool
	evaluation  frontend.Variable `gnark:"-"`
}

// ValueOf returns an Element[T] from a constant value.
// The input is converted to *big.Int and decomposed into limbs and packed into new Element[T].
func ValueOf[T FieldParams](constant interface{}) Element[T] {
	if constant == nil {
		r := newConstElement[T](0)
		return *r
	}
	r := newConstElement[T](constant)
	return *r
}

// newConstElement is shorthand for initialising new element using NewElement and
// taking pointer to it. We only want to have a public method for initialising
// an element which return a value because the user uses this only for witness
// creation and it mess up schema parsing.
func newConstElement[T FieldParams](v interface{}) *Element[T] {
	var fp T
	// convert to big.Int
	bValue := utils.FromInterface(v)

	// mod reduce
	if fp.Modulus().Cmp(&bValue) != 0 {
		bValue.Mod(&bValue, fp.Modulus())
	}

	// decompose into limbs
	// TODO @gbotrel use big.Int pool here
	blimbs := make([]*big.Int, fp.NbLimbs())
	for i := range blimbs {
		blimbs[i] = new(big.Int)
	}
	if err := decompose(&bValue, fp.BitsPerLimb(), blimbs); err != nil {
		panic(fmt.Errorf("decompose value: %w", err))
	}

	// assign limb values
	limbs := make([]frontend.Variable, len(blimbs))
	for i := range limbs {
		limbs[i] = frontend.Variable(blimbs[i])
	}
	return &Element[T]{
		Limbs:    limbs,
		overflow: 0,
		internal: true,
	}
}

// newInternalElement sets the limbs and overflow. Given as a function for later
// possible refactor.
func (f *Field[T]) newInternalElement(limbs []frontend.Variable, overflow uint) *Element[T] {
	return &Element[T]{Limbs: limbs, overflow: overflow, internal: true}
}

// GnarkInitHook describes how to initialise the element.
func (e *Element[T]) GnarkInitHook() {
	if e.Limbs == nil {
		*e = ValueOf[T](0)
		e.internal = false // we need to constrain in later.
	}
}

// copy makes a deep copy of the element.
func (e *Element[T]) copy() *Element[T] {
	r := Element[T]{}
	r.Limbs = make([]frontend.Variable, len(e.Limbs))
	copy(r.Limbs, e.Limbs)
	r.overflow = e.overflow
	r.internal = e.internal
	return &r
}
