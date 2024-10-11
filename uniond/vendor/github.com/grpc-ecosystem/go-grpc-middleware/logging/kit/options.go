package kit

import (
	"time"

	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
	grpc_logging "github.com/grpc-ecosystem/go-grpc-middleware/logging"
	"google.golang.org/grpc/codes"
)

var (
	defaultOptions = &options{
		shouldLog:       grpc_logging.DefaultDeciderMethod,
		codeFunc:        grpc_logging.DefaultErrorToCode,
		durationFunc:    DefaultDurationToField,
		timestampFormat: time.RFC3339,
	}
)

type options struct {
	levelFunc       CodeToLevel
	shouldLog       grpc_logging.Decider
	codeFunc        grpc_logging.ErrorToCode
	durationFunc    DurationToField
	timestampFormat string
}

type Option func(*options)

// CodeToLevel function defines the mapping between gRPC return codes and interceptor log level.
type CodeToLevel func(code codes.Code, logger log.Logger) log.Logger

// DurationToField function defines how to produce duration fields for logging
type DurationToField func(duration time.Duration) []interface{}

func evaluateServerOpt(opts []Option) *options {
	optCopy := &options{}
	*optCopy = *defaultOptions
	optCopy.levelFunc = DefaultCodeToLevel
	for _, o := range opts {
		o(optCopy)
	}
	return optCopy
}

func evaluateClientOpt(opts []Option) *options {
	optCopy := &options{}
	*optCopy = *defaultOptions
	optCopy.levelFunc = DefaultClientCodeToLevel
	for _, o := range opts {
		o(optCopy)
	}
	return optCopy
}

// WithDecider customizes the function for deciding if the gRPC interceptor logs should log.
func WithDecider(f grpc_logging.Decider) Option {
	return func(o *options) {
		o.shouldLog = f
	}
}

// WithLevels customizes the function for mapping gRPC return codes and interceptor log level statements.
func WithLevels(f CodeToLevel) Option {
	return func(o *options) {
		o.levelFunc = f
	}
}

// WithCodes customizes the function for mapping errors to error codes.
func WithCodes(f grpc_logging.ErrorToCode) Option {
	return func(o *options) {
		o.codeFunc = f
	}
}

// WithDurationField customizes the function for mapping request durations to log fields.
func WithDurationField(f DurationToField) Option {
	return func(o *options) {
		o.durationFunc = f
	}
}

// WithTimestampFormat customizes the timestamps emitted in the log fields.
func WithTimestampFormat(format string) Option {
	return func(o *options) {
		o.timestampFormat = format
	}
}

// DefaultCodeToLevel is the default implementation of gRPC return codes and interceptor log level for server side.
func DefaultCodeToLevel(code codes.Code, logger log.Logger) log.Logger {
	switch code {
	case codes.OK, codes.Canceled, codes.InvalidArgument, codes.NotFound, codes.AlreadyExists, codes.Unauthenticated:
		return level.Info(logger)
	case codes.DeadlineExceeded, codes.PermissionDenied, codes.ResourceExhausted, codes.FailedPrecondition, codes.Aborted, codes.OutOfRange, codes.Unavailable:
		return level.Warn(logger)
	case codes.Unknown, codes.Unimplemented, codes.Internal, codes.DataLoss:
		return level.Error(logger)
	default:
		return level.Error(logger)
	}
}

// DefaultClientCodeToLevel is the default implementation of gRPC return codes to log levels for client side.
func DefaultClientCodeToLevel(code codes.Code, logger log.Logger) log.Logger {
	switch code {
	case codes.OK, codes.Canceled, codes.InvalidArgument, codes.NotFound, codes.AlreadyExists, codes.ResourceExhausted, codes.FailedPrecondition, codes.Aborted, codes.OutOfRange:
		return level.Debug(logger)
	case codes.Unknown, codes.DeadlineExceeded, codes.PermissionDenied, codes.Unauthenticated:
		return level.Info(logger)
	case codes.Unimplemented, codes.Internal, codes.Unavailable, codes.DataLoss:
		return level.Warn(logger)
	default:
		return level.Info(logger)
	}
}

// DefaultDurationToField is the default implementation of converting request duration to a kit field.
var DefaultDurationToField = DurationToTimeMillisField

// DurationToTimeMillisField converts the duration to milliseconds and uses the key `grpc.time_ms`.
func DurationToTimeMillisField(duration time.Duration) []interface{} {
	return []interface{}{"grpc.time_ms", durationToMilliseconds(duration)}
}

// DurationToDurationField uses a Duration field to log the request duration
// and leaves it up to Log's encoder settings to determine how that is output.
func DurationToDurationField(duration time.Duration) []interface{} {
	return []interface{}{"grpc.duration", duration}
}

func durationToMilliseconds(duration time.Duration) float32 {
	return float32(duration.Nanoseconds()/1000) / 1000
}
