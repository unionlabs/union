package p

// old
type u1 int

// both
type A int
type u2 int

// simple type changes
// old
var (
	V1 string
	V3 A
	V7 <-chan int
)

// new
var (
	// i V1: changed from string to []string
	V1 []string
	V3 A // OK: same
	// i V7: changed from <-chan int to chan int
	V7 chan int
)

// interface type  changes
// old
var (
	V9  interface{ M() }
	V10 interface{ M() }
	V11 interface{ M() }
)

// new
var (
	// i V9: changed from interface{M()} to interface{}
	V9 interface{}
	// i V10: changed from interface{M()} to interface{M(); M2()}
	V10 interface {
		M2()
		M()
	}
	// i V11: changed from interface{M()} to interface{M(int)}
	V11 interface{ M(int) }
)

// struct type changes
// old
var (
	VS1 struct{ A, B int }
	VS2 struct{ A, B int }
	VS3 struct{ A, B int }
	VS4 struct {
		A int
		u1
	}
)

// new
var (
	// i VS1: changed from struct{A int; B int} to struct{B int; A int}
	VS1 struct{ B, A int }
	// i VS2: changed from struct{A int; B int} to struct{A int}
	VS2 struct{ A int }
	// i VS3: changed from struct{A int; B int} to struct{A int; B int; C int}
	VS3 struct{ A, B, C int }
	VS4 struct {
		A int
		u2
	}
)
