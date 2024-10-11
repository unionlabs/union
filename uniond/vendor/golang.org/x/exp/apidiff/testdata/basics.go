package p

// Same type in both: OK.
// both
type A int

// Changing the type is an incompatible change.
// old
type B int

// new
// i B: changed from int to string
type B string

// Adding a new type, whether alias or not, is a compatible change.
// new
// c AA: added
type AA = A

// c B1: added
type B1 bool

// Change of type for an unexported name doesn't matter...
// old
type t int

// new
type t string // OK: t isn't part of the API

// ...unless it is exposed.
// both
var V2 u

// old
type u string

// new
// i u: changed from string to int
type u int

// An exposed, unexported type can be renamed.
// both
type u2 int

// old
type u1 int

var V5 u1

// new
var V5 u2 // OK: V5 has changed type, but old u1 corresopnds to new u2

// Splitting a single type into two is an incompatible change.
// both
type u3 int

// old
type (
	Split1 = u1
	Split2 = u1
)

// new
type (
	Split1 = u2 // OK, since old u1 corresponds to new u2

	// This tries to make u1 correspond to u3
	// i Split2: changed from u1 to u3
	Split2 = u3
)

// Merging two types into one is OK.
// old
type (
	GoodMerge1 = u2
	GoodMerge2 = u3
)

// new
type (
	GoodMerge1 = u3
	GoodMerge2 = u3
)

// Merging isn't OK here because a method is lost.
// both
type u4 int

func (u4) M() {}

// old
type (
	BadMerge1 = u3
	BadMerge2 = u4
)

// new
type (
	BadMerge1 = u3
	// i u4.M: removed
	// What's really happening here is that old u4 corresponds to new u3,
	// and new u3's method set is not a superset of old u4's.
	BadMerge2 = u3
)

// old
type Rem int

// new
// i Rem: removed
