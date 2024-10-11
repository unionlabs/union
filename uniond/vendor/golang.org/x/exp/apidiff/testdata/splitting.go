package p

// Splitting types
//
// In the old world, there is one type with two names, one of which is an alias.
// In the new world, there are two distinct types.
//
// That is an incompatible change, because client code like
//
//     var v *T1 = new(T2)
//
// will succeed in the old world, where T1 and T2 name the same type,
// but fail in the new world.

// OK: in both old and new, A, B,  and C all name the same type.
// old
type (
	A = B
	B = C
	C int
)

// new
type (
	A = B
	B int
	C = A
)

// An example of splitting:

// Old has one type, D; new has E and D.
// both
type D int

// old
type E = D

// new
// i E: changed from D to E
type E D // old D corresponds with new E
// old D also corresponds with new D: problem

// Here we have a benign split.
// f and g are the same type in old and different types in new.
// But clients have no way of constructing an expression of type f,
// so they cannot write code that breaks.

// both
type f int

var Vg g // expose g

// old
type g = f

// new
// OK: f isn't exposed
type g f

// Here we have another incompatible split, even
// though the type names are unexported. The problem
// is that both names are exposed via exported variables.

// both
type h int

var Vj j // expose j
var Vh h // expose h

// old
type j = h

// new
// i Vj: changed from h to j
// e.g. p.Vj = p.Vh
type j h
