package i32

// Abs returns the absolute value of x.
func Abs(x int32) int32 {
	// Code adapted from https://bitbucket.org/SyntaxK/imath/src
	return x ^ x>>31 - x>>31
}
