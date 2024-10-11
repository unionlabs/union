package types

import "fmt"

// An Invariant is a function which tests a particular invariant.
// The invariant returns a descriptive message about what happened
// and a boolean indicating whether the invariant has been broken.
// The simulator will then halt and print the logs.
type Invariant func(ctx Context) (string, bool)

// Invariants defines a group of invariants
type Invariants []Invariant

// expected interface for registering invariants
type InvariantRegistry interface {
	RegisterRoute(moduleName, route string, invar Invariant)
}

// FormatInvariant returns a standardized invariant message.
func FormatInvariant(module, name, msg string) string {
	return fmt.Sprintf("%s: %s invariant\n%s\n", module, name, msg)
}
