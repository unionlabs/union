package errors

import (
	stdlib "errors"
	"fmt"
	"testing"

	"github.com/pkg/errors"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"
)

type errorsTestSuite struct {
	suite.Suite
}

func TestErrorsTestSuite(t *testing.T) {
	suite.Run(t, new(errorsTestSuite))
}

func (s *errorsTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *errorsTestSuite) TestCause() {
	std := stdlib.New("this is a stdlib error")

	cases := map[string]struct {
		err  error
		root error
	}{
		"Errors are self-causing": {
			err:  ErrUnauthorized,
			root: ErrUnauthorized,
		},
		"Wrap reveals root cause": {
			err:  Wrap(ErrUnauthorized, "foo"),
			root: ErrUnauthorized,
		},
		"Cause works for stderr as root": {
			err:  Wrap(std, "Some helpful text"),
			root: std,
		},
	}

	for testName, tc := range cases {
		s.Require().Equal(tc.root, errors.Cause(tc.err), testName)
	}
}

func (s *errorsTestSuite) TestErrorIs() {
	cases := map[string]struct {
		a      *Error
		b      error
		wantIs bool
	}{
		"instance of the same error": {
			a:      ErrUnauthorized,
			b:      ErrUnauthorized,
			wantIs: true,
		},
		"two different coded errors": {
			a:      ErrUnauthorized,
			b:      ErrOutOfGas,
			wantIs: false,
		},
		"successful comparison to a wrapped error": {
			a:      ErrUnauthorized,
			b:      Wrap(ErrUnauthorized, "gone"),
			wantIs: true,
		},
		"unsuccessful comparison to a wrapped error": {
			a:      ErrUnauthorized,
			b:      Wrap(ErrInsufficientFee, "too big"),
			wantIs: false,
		},
		"not equal to stdlib error": {
			a:      ErrUnauthorized,
			b:      fmt.Errorf("stdlib error"),
			wantIs: false,
		},
		"not equal to a wrapped stdlib error": {
			a:      ErrUnauthorized,
			b:      Wrap(fmt.Errorf("stdlib error"), "wrapped"),
			wantIs: false,
		},
		"nil is nil": {
			a:      nil,
			b:      nil,
			wantIs: true,
		},
		"nil is any error nil": {
			a:      nil,
			b:      (*customError)(nil),
			wantIs: true,
		},
		"nil is not not-nil": {
			a:      nil,
			b:      ErrUnauthorized,
			wantIs: false,
		},
		"not-nil is not nil": {
			a:      ErrUnauthorized,
			b:      nil,
			wantIs: false,
		},
	}
	for testName, tc := range cases {
		s.Require().Equal(tc.wantIs, tc.a.Is(tc.b), testName)
	}
}

func (s *errorsTestSuite) TestIsOf() {
	require := s.Require()

	var errNil *Error
	err := ErrInvalidAddress
	errW := Wrap(ErrLogic, "more info")

	require.False(IsOf(nil), "nil should always have no causer")
	require.False(IsOf(nil, err), "nil should always have no causer")
	require.False(IsOf(errNil), "nil error should always have no causer")
	require.False(IsOf(errNil, err), "nil error should always have no causer")

	require.False(IsOf(err))
	require.False(IsOf(err, nil))
	require.False(IsOf(err, ErrLogic))

	require.True(IsOf(errW, ErrLogic))
	require.True(IsOf(errW, err, ErrLogic))
	require.True(IsOf(errW, nil, errW), "error should much itself")
	err2 := errors.New("other error")
	require.True(IsOf(err2, nil, err2), "error should much itself")
}

type customError struct{}

func (customError) Error() string {
	return "custom error"
}

func (s *errorsTestSuite) TestWrapEmpty() {
	s.Require().Nil(Wrap(nil, "wrapping <nil>"))
}

func (s *errorsTestSuite) TestWrappedIs() {
	require := s.Require()
	err := Wrap(ErrTxTooLarge, "context")
	require.True(stdlib.Is(err, ErrTxTooLarge))

	err = Wrap(err, "more context")
	require.True(stdlib.Is(err, ErrTxTooLarge))

	err = Wrap(err, "even more context")
	require.True(stdlib.Is(err, ErrTxTooLarge))

	err = Wrap(ErrInsufficientFee, "...")
	require.False(stdlib.Is(err, ErrTxTooLarge))

	errs := stdlib.New("other")
	require.True(stdlib.Is(errs, errs))

	errw := &wrappedError{"msg", errs}
	require.True(errw.Is(errw), "should match itself")

	require.True(stdlib.Is(ErrInsufficientFee.Wrap("wrapped"), ErrInsufficientFee))
	require.True(IsOf(ErrInsufficientFee.Wrap("wrapped"), ErrInsufficientFee))
	require.True(stdlib.Is(ErrInsufficientFee.Wrapf("wrapped"), ErrInsufficientFee))
	require.True(IsOf(ErrInsufficientFee.Wrapf("wrapped"), ErrInsufficientFee))
}

func (s *errorsTestSuite) TestWrappedIsMultiple() {
	errTest := errors.New("test error")
	errTest2 := errors.New("test error 2")
	err := Wrap(errTest2, Wrap(errTest, "some random description").Error())
	s.Require().True(stdlib.Is(err, errTest2))
}

func (s *errorsTestSuite) TestWrappedIsFail() {
	errTest := errors.New("test error")
	errTest2 := errors.New("test error 2")
	err := Wrap(errTest2, Wrap(errTest, "some random description").Error())
	s.Require().False(stdlib.Is(err, errTest))
}

func (s *errorsTestSuite) TestWrappedUnwrap() {
	errTest := errors.New("test error")
	err := Wrap(errTest, "some random description")
	s.Require().Equal(errTest, stdlib.Unwrap(err))
}

func (s *errorsTestSuite) TestWrappedUnwrapMultiple() {
	errTest := errors.New("test error")
	errTest2 := errors.New("test error 2")
	err := Wrap(errTest2, Wrap(errTest, "some random description").Error())
	s.Require().Equal(errTest2, stdlib.Unwrap(err))
}

func (s *errorsTestSuite) TestWrappedUnwrapFail() {
	errTest := errors.New("test error")
	errTest2 := errors.New("test error 2")
	err := Wrap(errTest2, Wrap(errTest, "some random description").Error())
	s.Require().NotEqual(errTest, stdlib.Unwrap(err))
}

func (s *errorsTestSuite) TestABCIError() {
	s.Require().Equal("custom: tx parse error", ABCIError(testCodespace, 2, "custom").Error())
	s.Require().Equal("custom: unknown", ABCIError("unknown", 1, "custom").Error())
}

func (s *errorsTestSuite) TestGRPCStatus() {
	s.Require().Equal(codes.Unknown, grpcstatus.Code(errInternal))
	s.Require().Equal(codes.NotFound, grpcstatus.Code(ErrNotFound))

	status, ok := grpcstatus.FromError(ErrNotFound)
	s.Require().True(ok)
	s.Require().Equal("codespace testtesttest code 38: not found", status.Message())

	// test wrapping
	s.Require().Equal(codes.Unimplemented, grpcstatus.Code(ErrNotSupported.Wrap("test")))
	s.Require().Equal(codes.FailedPrecondition, grpcstatus.Code(ErrConflict.Wrapf("test %s", "foo")))

	status, ok = grpcstatus.FromError(ErrNotFound.Wrap("test"))
	s.Require().True(ok)
	s.Require().Equal("codespace testtesttest code 38: not found: test", status.Message())
}

const testCodespace = "testtesttest"

var (
	ErrTxDecode                = Register(testCodespace, 2, "tx parse error")
	ErrInvalidSequence         = Register(testCodespace, 3, "invalid sequence")
	ErrUnauthorized            = Register(testCodespace, 4, "unauthorized")
	ErrInsufficientFunds       = Register(testCodespace, 5, "insufficient funds")
	ErrUnknownRequest          = Register(testCodespace, 6, "unknown request")
	ErrInvalidAddress          = Register(testCodespace, 7, "invalid address")
	ErrInvalidPubKey           = Register(testCodespace, 8, "invalid pubkey")
	ErrUnknownAddress          = Register(testCodespace, 9, "unknown address")
	ErrInvalidCoins            = Register(testCodespace, 10, "invalid coins")
	ErrOutOfGas                = Register(testCodespace, 11, "out of gas")
	ErrInsufficientFee         = Register(testCodespace, 13, "insufficient fee")
	ErrTooManySignatures       = Register(testCodespace, 14, "maximum number of signatures exceeded")
	ErrNoSignatures            = Register(testCodespace, 15, "no signatures supplied")
	ErrJSONMarshal             = Register(testCodespace, 16, "failed to marshal JSON bytes")
	ErrJSONUnmarshal           = Register(testCodespace, 17, "failed to unmarshal JSON bytes")
	ErrInvalidRequest          = Register(testCodespace, 18, "invalid request")
	ErrMempoolIsFull           = Register(testCodespace, 20, "mempool is full")
	ErrTxTooLarge              = Register(testCodespace, 21, "tx too large")
	ErrKeyNotFound             = Register(testCodespace, 22, "key not found")
	ErrorInvalidSigner         = Register(testCodespace, 24, "tx intended signer does not match the given signer")
	ErrInvalidChainID          = Register(testCodespace, 28, "invalid chain-id")
	ErrInvalidType             = Register(testCodespace, 29, "invalid type")
	ErrUnknownExtensionOptions = Register(testCodespace, 31, "unknown extension options")
	ErrPackAny                 = Register(testCodespace, 33, "failed packing protobuf message to Any")
	ErrLogic                   = Register(testCodespace, 35, "internal logic error")
	ErrConflict                = RegisterWithGRPCCode(testCodespace, 36, codes.FailedPrecondition, "conflict")
	ErrNotSupported            = RegisterWithGRPCCode(testCodespace, 37, codes.Unimplemented, "feature not supported")
	ErrNotFound                = RegisterWithGRPCCode(testCodespace, 38, codes.NotFound, "not found")
	ErrIO                      = Register(testCodespace, 39, "Internal IO error")
)
