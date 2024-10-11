package glog

// runInAnotherModule is a simple wrapper that, being defined in another file,
// provides a different vmodule stack frame on the stack for use with
// glog.*Depth testing.
//
//go:noinline
func runInAnotherModule(f func() bool) bool {
	return f()
}
