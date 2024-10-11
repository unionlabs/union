package errors_test

import (
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/testutils"
)

func TestUnwrap(t *testing.T) {
	tt := testutils.T{t}

	e := fmt.Errorf("foo %w %w", fmt.Errorf("bar"), fmt.Errorf("baz"))

	// Compatibility with go 1.20: Unwrap() on a multierror returns nil
	// (per API documentation)
	tt.Check(errors.Unwrap(e) == nil)
}

// More detailed testing of Join is in datadriven_test.go. Here we make
// sure that the public API includes the stacktrace wrapper.
func TestJoin(t *testing.T) {
	e := errors.Join(errors.New("abc123"), errors.New("def456"))
	printed := fmt.Sprintf("%+v", e)
	expected := `Error types: (1) *withstack.withStack (2) *join.joinError (3) *withstack.withStack (4) *errutil.leafError (5) *withstack.withStack (6) *errutil.leafError`
	if !strings.Contains(printed, expected) {
		t.Errorf("Expected: %s to contain: %s", printed, expected)
	}
}
