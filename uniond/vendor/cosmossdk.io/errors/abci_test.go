package errors

import (
	"fmt"
	"io"
	"strings"
	"testing"

	"github.com/stretchr/testify/suite"
)

type abciTestSuite struct {
	suite.Suite
}

func TestABCITestSuite(t *testing.T) {
	suite.Run(t, new(abciTestSuite))
}

func (s *abciTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *abciTestSuite) TestABCInfo() {
	cases := map[string]struct {
		err       error
		debug     bool
		wantCode  uint32
		wantSpace string
		wantLog   string
	}{
		"plain SDK error": {
			err:       ErrUnauthorized,
			debug:     false,
			wantLog:   "unauthorized",
			wantCode:  ErrUnauthorized.code,
			wantSpace: testCodespace,
		},
		"wrapped SDK error": {
			err:       Wrap(Wrap(ErrUnauthorized, "foo"), "bar"),
			debug:     false,
			wantLog:   "bar: foo: unauthorized",
			wantCode:  ErrUnauthorized.code,
			wantSpace: testCodespace,
		},
		"nil is empty message": {
			err:       nil,
			debug:     false,
			wantLog:   "",
			wantCode:  0,
			wantSpace: "",
		},
		"nil SDK error is not an error": {
			err:       (*Error)(nil),
			debug:     false,
			wantLog:   "",
			wantCode:  0,
			wantSpace: "",
		},
		"stdlib returns error message in debug mode": {
			err:       io.EOF,
			debug:     true,
			wantLog:   "EOF",
			wantCode:  1,
			wantSpace: UndefinedCodespace,
		},
		// This is hard to test because of attached stacktrace. This
		// case is tested in an another test.
		// "wrapped stdlib is a full message in debug mode": {
		//	err:      Wrap(io.EOF, "cannot read file"),
		//	debug:    true,
		//	wantLog:  "cannot read file: EOF",
		//	wantCode: 1,
		// },
		"custom error": {
			err:       customErr{},
			debug:     false,
			wantLog:   "custom",
			wantCode:  999,
			wantSpace: "extern",
		},
		"custom error in debug mode": {
			err:       customErr{},
			debug:     true,
			wantLog:   "custom",
			wantCode:  999,
			wantSpace: "extern",
		},
	}

	for testName, tc := range cases {
		s.T().Run(testName, func(t *testing.T) {
			space, code, log := ABCIInfo(tc.err, tc.debug)
			s.Require().Equal(tc.wantSpace, space, testName)
			s.Require().Equal(tc.wantCode, code, testName)
			s.Require().Equal(tc.wantLog, log, testName)
		})
	}
}

func (s *abciTestSuite) TestABCIInfoStacktrace() {
	cases := map[string]struct {
		err            error
		debug          bool
		wantStacktrace bool
		wantErrMsg     string
	}{
		"wrapped SDK error in debug mode provides stacktrace": {
			err:            Wrap(ErrUnauthorized, "wrapped"),
			debug:          true,
			wantStacktrace: true,
			wantErrMsg:     "wrapped: unauthorized",
		},
		"wrapped SDK error in non-debug mode does not have stacktrace": {
			err:            Wrap(ErrUnauthorized, "wrapped"),
			debug:          false,
			wantStacktrace: false,
			wantErrMsg:     "wrapped: unauthorized",
		},
		"wrapped stdlib error in debug mode provides stacktrace": {
			err:            Wrap(fmt.Errorf("stdlib"), "wrapped"),
			debug:          true,
			wantStacktrace: true,
			wantErrMsg:     "wrapped: stdlib",
		},
	}

	const thisTestSrc = "cosmossdk.io/errors.(*abciTestSuite).TestABCIInfoStacktrace"

	for testName, tc := range cases {
		s.T().Run(testName, func(t *testing.T) {
			_, _, log := ABCIInfo(tc.err, tc.debug)
			if !tc.wantStacktrace {
				s.Require().Equal(tc.wantErrMsg, log, testName)
			} else {
				s.Require().True(strings.Contains(log, thisTestSrc), testName)
				s.Require().True(strings.Contains(log, tc.wantErrMsg), testName)
			}
		})
	}
}

func (s *abciTestSuite) TestABCIInfoHidesStacktrace() {
	err := Wrap(ErrUnauthorized, "wrapped")
	_, _, log := ABCIInfo(err, false)
	s.Require().Equal("wrapped: unauthorized", log)
}

func (s *abciTestSuite) TestABCIInfoSerializeErr() {
	var (
		// Create errors with stacktrace for equal comparison.
		myErrDecode = Wrap(ErrTxDecode, "test")
		myErrAddr   = Wrap(ErrInvalidAddress, "tester")
		myPanic     = ErrPanic
	)

	specs := map[string]struct {
		src   error
		debug bool
		exp   string
	}{
		"single error": {
			src:   myErrDecode,
			debug: false,
			exp:   "test: tx parse error",
		},
		"second error": {
			src:   myErrAddr,
			debug: false,
			exp:   "tester: invalid address",
		},
		"single error with debug": {
			src:   myErrDecode,
			debug: true,
			exp:   fmt.Sprintf("%+v", myErrDecode),
		},
		"do not redact in debug encoder": {
			src:   myPanic,
			debug: true,
			exp:   fmt.Sprintf("%+v", myPanic),
		},
	}
	for msg, spec := range specs {
		spec := spec
		_, _, log := ABCIInfo(spec.src, spec.debug)
		s.Require().Equal(spec.exp, log, msg)
	}
}

// customErr is a custom implementation of an error that provides an ABCICode
// method.
type customErr struct{}

func (customErr) Codespace() string { return "extern" }

func (customErr) ABCICode() uint32 { return 999 }

func (customErr) Error() string { return "custom" }
