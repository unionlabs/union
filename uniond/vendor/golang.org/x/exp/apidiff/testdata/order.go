package p

// apidiff's output (but not its correctness) could depend on the order
// in which declarations were visited.

// For example, this always correctly reported that U changed from T to U:

// old
type U = T

// new
// i U: changed from T to U
type U T

// both
type T int

// But the test below, which is just a renaming of the one above, would report
// that B was changed from B to B! apidiff was not wrong--there is an
// incompatibility here--but it expressed itself poorly.
//
// This happened because old.A was processed first (Scope.Names returns names
// sorted), resulting in old.B corresponding with new.A. Later, when old.B
// was processed, it was matched with new.B. But since there was already a
// correspondence for old.B, the blame for the incompatibility was put on new.B.

// The fix is to establish the correspondence between same-named types first.

// old
type A = B

// new
// i A: changed from B to A
type A B

// both
type B int
