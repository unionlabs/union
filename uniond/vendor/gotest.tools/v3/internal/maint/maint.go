// Package maint implements assert.TestingT for uses outside of test cases,
// for example, in a TestMain.
package maint // import "gotest.tools/v3/internal/maint"

import (
	"fmt"
	"os"
)

// T provides an implementation of assert.TestingT which uses os.Exit, and
// fmt.Println. This implementation can be used outside of test cases to provide
// assert.TestingT, for example in a TestMain.
var T = t{}

type t struct{}

// FailNow exits with a non-zero code
func (t t) FailNow() {
	os.Exit(1)
}

// Fail exits with a non-zero code
func (t t) Fail() {
	os.Exit(2)
}

// Log args by printing them to stdout
func (t t) Log(args ...interface{}) {
	fmt.Println(args...)
}
