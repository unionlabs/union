package issue68744

import "fmt"

// The use of "any" here is crucial to exercise the bug.
// (None of our earlier tests covered this vital detail!)
func wrapf(format string, args ...any) { // want wrapf:"printfWrapper"
	fmt.Printf(format, args...)
}

func _() {
	wrapf("%s", 123) // want `issue68744.wrapf format %s has arg 123 of wrong type int`
}
