package errors

import (
	"fmt"
	"reflect"

	"github.com/pkg/errors"
	grpccodes "google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"
)

// UndefinedCodespace when we explicitly declare no codespace
const UndefinedCodespace = "undefined"

var (
	// errInternal should never be exposed, but we reserve this code for non-specified errors
	errInternal = Register(UndefinedCodespace, 1, "internal")

	// ErrStopIterating is used to break out of an iteration
	ErrStopIterating = Register(UndefinedCodespace, 2, "stop iterating")

	// ErrPanic should only be set when we recovering from a panic
	ErrPanic = Register(UndefinedCodespace, 111222, "panic")
)

// Register returns an error instance that should be used as the base for
// creating error instances during runtime.
//
// Popular root errors are declared in this package, but extensions may want to
// declare custom codes. This function ensures that no error code is used
// twice. Attempt to reuse an error code results in panic.
//
// Use this function only during a program startup phase.
func Register(codespace string, code uint32, description string) *Error {
	return RegisterWithGRPCCode(codespace, code, grpccodes.Unknown, description)
}

// RegisterWithGRPCCode is a version of Register that associates a gRPC error
// code with a registered error.
func RegisterWithGRPCCode(codespace string, code uint32, grpcCode grpccodes.Code, description string) *Error {
	// TODO - uniqueness is (codespace, code) combo
	if e := getUsed(codespace, code); e != nil {
		panic(fmt.Sprintf("error with code %d is already registered: %q", code, e.desc))
	}

	err := &Error{codespace: codespace, code: code, desc: description, grpcCode: grpcCode}
	setUsed(err)

	return err
}

// usedCodes is keeping track of used codes to ensure their uniqueness. No two
// error instances should share the same (codespace, code) tuple.
var usedCodes = map[string]*Error{}

func errorID(codespace string, code uint32) string {
	return fmt.Sprintf("%s:%d", codespace, code)
}

func getUsed(codespace string, code uint32) *Error {
	return usedCodes[errorID(codespace, code)]
}

func setUsed(err *Error) {
	usedCodes[errorID(err.codespace, err.code)] = err
}

// ABCIError will resolve an error code/log from an abci result into
// an error message. If the code is registered, it will map it back to
// the canonical error, so we can do eg. ErrNotFound.Is(err) on something
// we get back from an external API.
//
// This should *only* be used in clients, not in the server side.
// The server (abci app / blockchain) should only refer to registered errors
func ABCIError(codespace string, code uint32, log string) error {
	if e := getUsed(codespace, code); e != nil {
		return Wrap(e, log)
	}
	// This is a unique error, will never match on .Is()
	// Use Wrap here to get a stack trace
	return Wrap(&Error{codespace: codespace, code: code, desc: "unknown"}, log)
}

// Error represents a root error.
//
// Weave framework is using root error to categorize issues. Each instance
// created during the runtime should wrap one of the declared root errors. This
// allows error tests and returning all errors to the client in a safe manner.
//
// All popular root errors are declared in this package. If an extension has to
// declare a custom root error, always use Register function to ensure
// error code uniqueness.
type Error struct {
	codespace string
	code      uint32
	desc      string
	grpcCode  grpccodes.Code
}

// New is an alias for Register.
func New(codespace string, code uint32, desc string) *Error {
	return Register(codespace, code, desc)
}

func (e Error) Error() string {
	return e.desc
}

func (e Error) ABCICode() uint32 {
	return e.code
}

func (e Error) Codespace() string {
	return e.codespace
}

// Is check if given error instance is of a given kind/type. This involves
// unwrapping given error using the Cause method if available.
func (e *Error) Is(err error) bool {
	// Reflect usage is necessary to correctly compare with
	// a nil implementation of an error.
	if e == nil {
		return isNilErr(err)
	}

	for {
		if err == e {
			return true
		}

		// If this is a collection of errors, this function must return
		// true if at least one from the group match.
		if u, ok := err.(unpacker); ok {
			for _, er := range u.Unpack() {
				if e.Is(er) {
					return true
				}
			}
		}

		if c, ok := err.(causer); ok {
			err = c.Cause()
		} else {
			return false
		}
	}
}

// Wrap extends this error with an additional information.
// It's a handy function to call Wrap with sdk errors.
func (e *Error) Wrap(desc string) error { return Wrap(e, desc) }

// Wrapf extends this error with an additional information.
// It's a handy function to call Wrapf with sdk errors.
func (e *Error) Wrapf(desc string, args ...interface{}) error { return Wrapf(e, desc, args...) }

func (e *Error) GRPCStatus() *grpcstatus.Status {
	return grpcstatus.Newf(e.grpcCode, "codespace %s code %d: %s", e.codespace, e.code, e.desc)
}

func isNilErr(err error) bool {
	// Reflect usage is necessary to correctly compare with
	// a nil implementation of an error.
	if err == nil {
		return true
	}
	if reflect.ValueOf(err).Kind() == reflect.Struct {
		return false
	}
	return reflect.ValueOf(err).IsNil()
}

// Wrap extends given error with an additional information.
//
// If the wrapped error does not provide ABCICode method (ie. stdlib errors),
// it will be labeled as internal error.
//
// If err is nil, this returns nil, avoiding the need for an if statement when
// wrapping a error returned at the end of a function
func Wrap(err error, description string) error {
	if err == nil {
		return nil
	}

	// If this error does not carry the stacktrace information yet, attach
	// one. This should be done only once per error at the lowest frame
	// possible (most inner wrap).
	if stackTrace(err) == nil {
		err = errors.WithStack(err)
	}

	return &wrappedError{
		parent: err,
		msg:    description,
	}
}

// Wrapf extends given error with an additional information.
//
// This function works like Wrap function with additional functionality of
// formatting the input as specified.
func Wrapf(err error, format string, args ...interface{}) error {
	desc := fmt.Sprintf(format, args...)
	return Wrap(err, desc)
}

type wrappedError struct {
	// This error layer description.
	msg string
	// The underlying error that triggered this one.
	parent error
}

func (e *wrappedError) Error() string {
	return fmt.Sprintf("%s: %s", e.msg, e.parent.Error())
}

func (e *wrappedError) Cause() error {
	return e.parent
}

// Is reports whether any error in e's chain matches a target.
func (e *wrappedError) Is(target error) bool {
	if e == target {
		return true
	}

	w := e.Cause()
	for {
		if w == target {
			return true
		}

		x, ok := w.(causer)
		if ok {
			w = x.Cause()
		}
		if x == nil {
			return false
		}
	}
}

// Unwrap implements the built-in errors.Unwrap
func (e *wrappedError) Unwrap() error {
	return e.parent
}

// GRPCStatus gets the gRPC status from the wrapped error or returns an unknown gRPC status.
func (e *wrappedError) GRPCStatus() *grpcstatus.Status {
	w := e.Cause()
	for {
		if hasStatus, ok := w.(interface {
			GRPCStatus() *grpcstatus.Status
		}); ok {
			status := hasStatus.GRPCStatus()
			return grpcstatus.New(status.Code(), fmt.Sprintf("%s: %s", status.Message(), e.msg))
		}

		x, ok := w.(causer)
		if ok {
			w = x.Cause()
		}
		if x == nil {
			return grpcstatus.New(grpccodes.Unknown, e.msg)
		}
	}
}

// Recover captures a panic and stop its propagation. If panic happens it is
// transformed into a ErrPanic instance and assigned to given error. Call this
// function using defer in order to work as expected.
func Recover(err *error) {
	if r := recover(); r != nil {
		*err = Wrapf(ErrPanic, "%v", r)
	}
}

// WithType is a helper to augment an error with a corresponding type message
func WithType(err error, obj interface{}) error {
	return Wrap(err, fmt.Sprintf("%T", obj))
}

// IsOf checks if a received error is caused by one of the target errors.
// It extends the errors.Is functionality to a list of errors.
func IsOf(received error, targets ...error) bool {
	if received == nil {
		return false
	}
	for _, t := range targets {
		if errors.Is(received, t) {
			return true
		}
	}
	return false
}

// causer is an interface implemented by an error that supports wrapping. Use
// it to test if an error wraps another error instance.
type causer interface {
	Cause() error
}

type unpacker interface {
	Unpack() []error
}
