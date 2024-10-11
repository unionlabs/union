// These tests demonstrate the correct handling of symbols
// in packages other than two being compared.
// See the lines in establishCorrespondence beginning
//
//	if newn, ok := new.(*types.Named); ok
package p

// both

// gofmt insists on grouping imports, so old and new
// must both have both imports.
import (
	"io"
	"text/tabwriter"
)

// Here we have two named types in different packages.
// They have the same package-relative name, but we compare
// the package-qualified names.

// old
var V io.Writer
var _ tabwriter.Writer

// new
// i V: changed from io.Writer to text/tabwriter.Writer
var V tabwriter.Writer
var _ io.Writer

// Here one type is a basic type.
// Example from https://go.dev/issue/61385.
// apidiff would previously report
//	 F2: changed from func(io.ReadCloser) to func(io.ReadCloser)
//   io.ReadCloser: changed from interface{io.Reader; io.Closer} to int

// old
func F1(io.ReadCloser) {}

// new
// i F1: changed from func(io.ReadCloser) to func(int)
func F1(int) {}

// both
func F2(io.ReadCloser) {}
