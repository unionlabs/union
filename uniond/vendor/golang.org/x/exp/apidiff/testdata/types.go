package p

// both
import "io"

// old
type u1 int

// both
type u2 int

// old
const C5 = 3

type (
	A1 [1]int
	A2 [2]int
	A3 [C5]int
)

// new
// i C5: value changed from 3 to 4
const C5 = 4

type (
	A1 [1]int
	// i A2: changed from [2]int to [2]bool
	A2 [2]bool
	// i A3: changed from [3]int to [4]int
	A3 [C5]int
)

// old
type (
	Sl []int
	P1 *int
	P2 *u1
)

// new
type (
	// i Sl: changed from []int to []string
	Sl []string
	// i P1: changed from *int to **bool
	P1 **bool
	P2 *u2 // OK: u1 corresponds to u2
)

// old
type Bc1 int32
type Bc2 uint
type Bc3 float32
type Bc4 complex64

// new
// c Bc1: changed from int32 to int
type Bc1 int

// c Bc2: changed from uint to uint64
type Bc2 uint64

// c Bc3: changed from float32 to float64
type Bc3 float64

// c Bc4: changed from complex64 to complex128
type Bc4 complex128

// old
type Bi1 int32
type Bi2 uint
type Bi3 float64
type Bi4 complex128

// new
// i Bi1: changed from int32 to int16
type Bi1 int16

// i Bi2: changed from uint to uint32
type Bi2 uint32

// i Bi3: changed from float64 to float32
type Bi3 float32

// i Bi4: changed from complex128 to complex64
type Bi4 complex64

// old
type (
	M1 map[string]int
	M2 map[string]int
	M3 map[string]int
)

// new
type (
	M1 map[string]int
	// i M2: changed from map[string]int to map[int]int
	M2 map[int]int
	// i M3: changed from map[string]int to map[string]string
	M3 map[string]string
)

// old
type (
	Ch1 chan int
	Ch2 <-chan int
	Ch3 chan int
	Ch4 <-chan int
)

// new
type (
	// i Ch1, element type: changed from int to bool
	Ch1 chan bool
	// i Ch2: changed direction
	Ch2 chan<- int
	// i Ch3: changed direction
	Ch3 <-chan int
	// c Ch4: removed direction
	Ch4 chan int
)

// old
type I1 interface {
	M1()
	M2()
}

// new
type I1 interface {
	// M1()
	// i I1.M1: removed
	M2(int)
	// i I1.M2: changed from func() to func(int)
	M3()
	// i I1.M3: added
	m()
	// i I1.m: added unexported method
}

// old
type I2 interface {
	M1()
	m()
}

// new
type I2 interface {
	M1()
	// m() Removing an unexported method is OK.
	m2() // OK, because old already had an unexported method
	// c I2.M2: added
	M2()
}

// old
type I3 interface {
	io.Reader
	M()
}

// new
// OK: what matters is the method set; the name of the embedded
// interface isn't important.
type I3 interface {
	M()
	Read([]byte) (int, error)
}

// old
type I4 io.Writer

// new
// OK: in both, I4 is a distinct type from io.Writer, and
// the old and new I4s have the same method set.
type I4 interface {
	Write([]byte) (int, error)
}

// old
type I5 = io.Writer

// new
// i I5: changed from io.Writer to I5
// In old, I5 and io.Writer are the same type; in new,
// they are different. That can break something like:
//
//	var _ func(io.Writer) = func(pkg.I6) {}
type I5 io.Writer

// old
type I6 interface{ Write([]byte) (int, error) }

// new
// i I6: changed from I6 to io.Writer
// Similar to the above.
type I6 = io.Writer

//// correspondence with a basic type
// Basic types are technically defined types, but they aren't
// represented that way in go/types, so the cases below are special.

// both
type T1 int

// old
var VT1 T1

// new
// i VT1: changed from T1 to int
// This fails because old T1 corresponds to both int and new T1.
var VT1 int

// old
type t2 int

var VT2 t2

// new
// OK: t2 corresponds to int. It's fine that old t2
// doesn't exist in new.
var VT2 int

// both
type t3 int

func (t3) M() {}

// old
var VT3 t3

// new
// i t3.M: removed
// Here the change from t3 to int is incompatible
// because old t3 has an exported method.
var VT3 int

// old
var VT4 int

// new
type t4 int

// i VT4: changed from int to t4
// This is incompatible because of code like
//
//	VT4 + int(1)
//
// which works in old but fails in new.
// The difference from the above cases is that
// in those, we were merging two types into one;
// here, we are splitting int into t4 and int.
var VT4 t4
