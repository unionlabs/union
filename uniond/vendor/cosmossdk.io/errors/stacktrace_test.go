package errors

import (
	"errors"
	"fmt"
	"reflect"
	"strings"
)

func (s *errorsTestSuite) TestStackTrace() {
	cases := map[string]struct {
		err       error
		wantError string
	}{
		"New gives us a stacktrace": {
			err:       Wrap(ErrNoSignatures, "name"),
			wantError: "name: no signatures supplied",
		},
		"Wrapping stderr gives us a stacktrace": {
			err:       Wrap(fmt.Errorf("foo"), "standard"),
			wantError: "standard: foo",
		},
		"Wrapping pkg/errors gives us clean stacktrace": {
			err:       Wrap(errors.New("bar"), "pkg"),
			wantError: "pkg: bar",
		},
		"Wrapping inside another function is still clean": {
			err:       Wrap(fmt.Errorf("indirect"), "do the do"),
			wantError: "do the do: indirect",
		},
	}

	// Wrapping code is unwanted in the errors stack trace.
	unwantedSrc := []string{
		"cosmossdk.io/errors.Wrap\n",
		"cosmossdk.io/errors.Wrapf\n",
		"runtime.goexit\n",
	}
	const thisTestSrc = "errors/stacktrace_test.go"

	for _, tc := range cases {
		s.Require().True(reflect.DeepEqual(tc.err.Error(), tc.wantError))
		s.Require().NotNil(stackTrace(tc.err))
		fullStack := fmt.Sprintf("%+v", tc.err)
		s.Require().True(strings.Contains(fullStack, thisTestSrc))
		s.Require().True(strings.Contains(fullStack, tc.wantError))

		for _, src := range unwantedSrc {
			if strings.Contains(fullStack, src) {
				s.T().Logf("Stack trace below\n----%s\n----", fullStack)
				s.T().Logf("full stack contains unwanted source file path: %q", src)
			}
		}

		tinyStack := fmt.Sprintf("%v", tc.err)
		s.Require().True(strings.HasPrefix(tinyStack, tc.wantError))
		s.Require().False(strings.Contains(tinyStack, "\n"))
		// contains a link to where it was created, which must
		// be here, not the Wrap() function
		s.Require().True(strings.Contains(tinyStack, thisTestSrc))
	}
}
