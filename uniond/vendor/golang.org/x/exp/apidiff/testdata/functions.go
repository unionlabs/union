package p

// old
type u1 int

// both
type A int
type u2 int

// new
// c AA: added
type AA = A

// old
func F1(a int, b string) map[u1]A { return nil }
func F2(int)                      {}
func F3(int)                      {}
func F4(int) int                  { return 0 }
func F5(int) int                  { return 0 }
func F6(int)                      {}
func F7(interface{})              {}

// new
func F1(c int, d string) map[u2]AA { return nil } //OK: same (since u1 corresponds to u2)

// i F2: changed from func(int) to func(int) bool
func F2(int) bool { return true }

// i F3: changed from func(int) to func(int, int)
func F3(int, int) {}

// i F4: changed from func(int) int to func(bool) int
func F4(bool) int { return 0 }

// i F5: changed from func(int) int to func(int) string
func F5(int) string { return "" }

// i F6: changed from func(int) to func(...int)
func F6(...int) {}

// i F7: changed from func(interface{}) to func(interface{x()})
func F7(a interface{ x() }) {}

// old
func F8(bool) {}

// new
// c F8: changed from func to var
var F8 func(bool)

// old
var F9 func(int)

// new
// i F9: changed from var to func
func F9(int) {}

// both
// OK, even though new S is incompatible with old S (see below)
func F10(S) {}

// old
type S struct {
	A int
}

// new
type S struct {
	// i S.A: changed from int to bool
	A bool
}
