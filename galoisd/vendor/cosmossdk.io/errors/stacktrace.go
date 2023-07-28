package errors

import (
	"fmt"
	"io"
	"runtime"
	"strings"

	"github.com/pkg/errors"
)

func matchesFunc(f errors.Frame, prefixes ...string) bool {
	fn := funcName(f)
	for _, prefix := range prefixes {
		if strings.HasPrefix(fn, prefix) {
			return true
		}
	}
	return false
}

// funcName returns the name of this function, if known.
func funcName(f errors.Frame) string {
	// this looks a bit like magic, but follows example here:
	// https://github.com/pkg/errors/blob/v0.8.1/stack.go#L43-L50
	pc := uintptr(f) - 1
	fn := runtime.FuncForPC(pc)
	if fn == nil {
		return "unknown"
	}
	return fn.Name()
}

func fileLine(f errors.Frame) (string, int) {
	// this looks a bit like magic, but follows example here:
	// https://github.com/pkg/errors/blob/v0.8.1/stack.go#L14-L27
	// as this is where we get the Frames
	pc := uintptr(f) - 1
	fn := runtime.FuncForPC(pc)
	if fn == nil {
		return "unknown", 0
	}
	return fn.FileLine(pc)
}

func trimInternal(st errors.StackTrace) errors.StackTrace {
	// trim our internal parts here
	// manual error creation, or runtime for caught panics
	for matchesFunc(st[0],
		// where we create errors
		"cosmossdk.io/errors.Wrap",
		"cosmossdk.io/errors.Wrapf",
		"cosmossdk.io/errors.WithType",
		// runtime are added on panics
		"runtime.",
		// _test is defined in coverage tests, causing failure
		// "/_test/"
	) {
		st = st[1:]
	}
	// trim out outer wrappers (runtime.goexit and test library if present)
	for l := len(st) - 1; l > 0 && matchesFunc(st[l], "runtime.", "testing."); l-- {
		st = st[:l]
	}
	return st
}

func writeSimpleFrame(s io.Writer, f errors.Frame) {
	file, line := fileLine(f)
	// cut file at "github.com/"
	// TODO: generalize better for other hosts?
	chunks := strings.SplitN(file, "github.com/", 2)
	if len(chunks) == 2 {
		file = chunks[1]
	}
	_, _ = fmt.Fprintf(s, " [%s:%d]", file, line)
}

// Format works like pkg/errors, with additions.
// %s is just the error message
// %+v is the full stack trace
// %v appends a compressed [filename:line] where the error
//    was created
//
// Inspired by https://github.com/pkg/errors/blob/v0.8.1/errors.go#L162-L176
func (e *wrappedError) Format(s fmt.State, verb rune) {
	// normal output here....
	if verb != 'v' {
		_, _ = fmt.Fprint(s, e.Error())
		return
	}
	// work with the stack trace... whole or part
	stack := trimInternal(stackTrace(e))
	if s.Flag('+') {
		_, _ = fmt.Fprintf(s, "%+v\n", stack)
		_, _ = fmt.Fprint(s, e.Error())
	} else {
		_, _ = fmt.Fprint(s, e.Error())
		writeSimpleFrame(s, stack[0])
	}
}

// stackTrace returns the first found stack trace frame carried by given error
// or any wrapped error. It returns nil if no stack trace is found.
func stackTrace(err error) errors.StackTrace {
	type stackTracer interface {
		StackTrace() errors.StackTrace
	}

	for {
		if st, ok := err.(stackTracer); ok {
			return st.StackTrace()
		}

		if c, ok := err.(causer); ok {
			err = c.Cause()
		} else {
			return nil
		}
	}
}
