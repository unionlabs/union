package p

// old
type S1 struct {
	A int
	B string
	C bool
	d float32
}

// new
type S1 = s1

type s1 struct {
	C chan int
	// i S1.C: changed from bool to chan int
	A int
	// i S1.B: removed
	// i S1: old is comparable, new is not
	x []int
	d float32
	E bool
	// c S1.E: added
}

// old
type embed struct {
	E string
}

type S2 struct {
	A int
	embed
}

// new
type embedx struct {
	E string
}

type S2 struct {
	embedx // OK: the unexported embedded field changed names, but the exported field didn't
	A      int
}

// both
type F int

// old
type S3 struct {
	A int
	embed
}

// new
type embed struct{ F int }

type S3 struct {
	// i S3.E: removed
	embed
	// c S3.F: added
	A int
}

// both
type A1 [1]int

// old
type embed2 struct {
	embed3
	F // shadows embed3.F
}

type embed3 struct {
	F bool
}

type alias = struct{ D bool }

type S4 struct {
	int
	*embed2
	embed
	E int // shadows embed.E
	alias
	A1
	*S4
}

// new
type S4 struct {
	// OK: removed unexported fields
	// D and F marked as added because they are now part of the immediate fields
	D bool
	// c S4.D: added
	E int // OK: same as in old
	F F
	// c S4.F: added
	A1  // OK: same
	*S4 // OK: same (recursive embedding)
}

// Difference between exported selectable fields and exported immediate fields.
// both
type S5 struct{ A int }

// old
// Exported immediate fields: A, S5
// Exported selectable fields: A int, S5 S5
type S6 struct {
	S5 S5
	A  int
}

// new
// Exported immediate fields: S5
// Exported selectable fields: A int, S5 S5.

// i S6.A: removed
type S6 struct {
	S5
}

// Ambiguous fields can exist; they just can't be selected.
// both
type (
	embed7a struct{ E int }
	embed7b struct{ E bool }
)

// old
type S7 struct { // legal, but no selectable fields
	embed7a
	embed7b
}

// new
type S7 struct {
	embed7a
	embed7b
	// c S7.E: added
	E string
}
