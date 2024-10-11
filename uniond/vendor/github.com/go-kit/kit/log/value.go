package log

import (
	"time"

	"github.com/go-kit/log"
)

// A Valuer generates a log value. When passed to With, WithPrefix, or
// WithSuffix in a value element (odd indexes), it represents a dynamic
// value which is re-evaluated with each log event.
type Valuer = log.Valuer

// Timestamp returns a timestamp Valuer. It invokes the t function to get the
// time; unless you are doing something tricky, pass time.Now.
//
// Most users will want to use DefaultTimestamp or DefaultTimestampUTC, which
// are TimestampFormats that use the RFC3339Nano format.
func Timestamp(t func() time.Time) Valuer {
	return log.Timestamp(t)
}

// TimestampFormat returns a timestamp Valuer with a custom time format. It
// invokes the t function to get the time to format; unless you are doing
// something tricky, pass time.Now. The layout string is passed to
// Time.Format.
//
// Most users will want to use DefaultTimestamp or DefaultTimestampUTC, which
// are TimestampFormats that use the RFC3339Nano format.
func TimestampFormat(t func() time.Time, layout string) Valuer {
	return log.TimestampFormat(t, layout)
}

// Caller returns a Valuer that returns a file and line from a specified depth
// in the callstack. Users will probably want to use DefaultCaller.
func Caller(depth int) Valuer {
	return log.Caller(depth)
}

var (
	// DefaultTimestamp is a Valuer that returns the current wallclock time,
	// respecting time zones, when bound.
	DefaultTimestamp = log.DefaultTimestamp

	// DefaultTimestampUTC is a Valuer that returns the current time in UTC
	// when bound.
	DefaultTimestampUTC = log.DefaultTimestampUTC

	// DefaultCaller is a Valuer that returns the file and line where the Log
	// method was invoked. It can only be used with log.With.
	DefaultCaller = log.DefaultCaller
)
