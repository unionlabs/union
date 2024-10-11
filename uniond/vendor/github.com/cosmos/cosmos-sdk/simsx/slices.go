package simsx

// Collect applies the function f to each element in the source slice,
// returning a new slice containing the results.
//
// The source slice can contain elements of any type T, and the function f
// should take an element of type T as input and return a value of any type E.
//
// Example usage:
//
//	source := []int{1, 2, 3, 4, 5}
//	double := Collect(source, func(x int) int {
//	    return x * 2
//	})
//	// double is now []int{2, 4, 6, 8, 10}
func Collect[T, E any](source []T, f func(a T) E) []E {
	r := make([]E, len(source))
	for i, v := range source {
		r[i] = f(v)
	}
	return r
}

// First returns the first element in the slice that matches the condition
func First[T any](source []T, f func(a T) bool) *T {
	for i := 0; i < len(source); i++ {
		if f(source[i]) {
			return &source[i]
		}
	}
	return nil
}

// OneOf returns a random element from the given slice using the provided random number generator.
// Panics for empty or nil slice
func OneOf[T any](r interface{ Intn(n int) int }, s []T) T {
	return s[r.Intn(len(s))]
}
