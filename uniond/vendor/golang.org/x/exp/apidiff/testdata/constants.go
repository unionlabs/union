package p

// old
type u1 int

// both
type u2 int

// type changes
// old

const (
	C1     = 1
	C2 int = 2
	C3     = 3
	C4 u1  = 4
)

var V8 int

// new
const (
	// i C1: changed from untyped int to untyped string
	C1 = "1"
	// i C2: changed from int to untyped int
	C2 = -1
	// i C3: changed from untyped int to int
	C3 int = 3
	// i V8: changed from var to const
	V8 int = 1
	C4 u2  = 4 // OK: u1 corresponds to u2
)

// value change
// old
const (
	Cr1 = 1
	Cr2 = "2"
	Cr3 = 3.5
	Cr4 = complex(0, 4.1)
)

// new
const (
	// i Cr1: value changed from 1 to -1
	Cr1 = -1
	// i Cr2: value changed from "2" to "3"
	Cr2 = "3"
	// i Cr3: value changed from 3.5 to 3.8
	Cr3 = 3.8
	// i Cr4: value changed from (0 + 4.1i) to (4.1 + 0i)
	Cr4 = complex(4.1, 0)
)
