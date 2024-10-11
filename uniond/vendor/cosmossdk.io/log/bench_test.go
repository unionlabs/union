package log_test

import (
	"bytes"
	"errors"
	"io"
	"testing"
	"time"

	"github.com/rs/zerolog"

	"cosmossdk.io/log"
)

const message = "test message"

func BenchmarkLoggers(b *testing.B) {
	b.ReportAllocs()

	type benchCase struct {
		name    string
		keyVals []any
	}

	// Just test two simple cases for the nop logger benchmarks.
	nopCases := []benchCase{
		{name: "empty key vals"},
		{name: "single string", keyVals: []any{"foo", "bar"}},
	}

	benchCases := append(nopCases, []benchCase{
		{
			name:    "single small int",
			keyVals: []any{"foo", 1},
		},
		{
			// Small numbers may be optimized, so check if an unusual/larger number performs different.
			name:    "single largeish int",
			keyVals: []any{"foo", 123456789},
		},
		{
			name:    "single float",
			keyVals: []any{"foo", 2.71828182},
		},
		{
			name:    "single byte slice",
			keyVals: []any{"foo", []byte{0xde, 0xad, 0xbe, 0xef}},
		},
		{
			name:    "single duration",
			keyVals: []any{"foo", 10 * time.Second},
		},

		{
			name:    "two values",
			keyVals: []any{"foo", "foo", "bar", "bar"},
		},
		{
			name:    "four values",
			keyVals: []any{"foo", "foo", "bar", "bar", "baz", "baz", "quux", "quux"},
		},
		{
			name:    "eight values",
			keyVals: []any{"one", 1, "two", 2, "three", 3, "four", 4, "five", 5, "six", 6, "seven", 7, "eight", 8},
		},
	}...)

	// If running with "go test -v", print out the log messages as a sanity check.
	if testing.Verbose() {
		checkBuf := new(bytes.Buffer)
		for _, bc := range benchCases {
			checkBuf.Reset()
			zl := zerolog.New(checkBuf)
			logger := log.NewCustomLogger(zl)
			logger.Info(message, bc.keyVals...)

			b.Logf("zero logger output for %s: %s", bc.name, checkBuf.String())
		}
	}

	// The real logger exposed by this package,
	// writing to an io.Discard writer,
	// so that real write time is negligible.
	b.Run("zerolog", func(b *testing.B) {
		for _, bc := range benchCases {
			bc := bc
			b.Run(bc.name, func(b *testing.B) {
				zl := zerolog.New(io.Discard)
				logger := log.NewCustomLogger(zl)

				for i := 0; i < b.N; i++ {
					logger.Info(message, bc.keyVals...)
				}
			})
		}
	})

	// The nop logger we use expose in the public API,
	// also useful as a reference for how expensive zerolog is.
	b.Run("specialized nop logger", func(b *testing.B) {
		for _, bc := range nopCases {
			bc := bc
			b.Run(bc.name, func(b *testing.B) {
				logger := log.NewNopLogger()

				for i := 0; i < b.N; i++ {
					logger.Info(message, bc.keyVals...)
				}
			})
		}
	})

	// To compare with the custom nop logger.
	// The zerolog wrapper is about 1/3 the speed of the specialized nop logger,
	// so we offer the specialized version in the exported API.
	b.Run("zerolog nop logger", func(b *testing.B) {
		for _, bc := range nopCases {
			bc := bc
			b.Run(bc.name, func(b *testing.B) {
				logger := log.NewCustomLogger(zerolog.Nop())

				for i := 0; i < b.N; i++ {
					logger.Info(message, bc.keyVals...)
				}
			})
		}
	})
}

func BenchmarkLoggers_StructuredVsFields(b *testing.B) {
	b.ReportAllocs()

	errorToLog := errors.New("error")
	byteSliceToLog := []byte{0xde, 0xad, 0xbe, 0xef}

	b.Run("logger structured", func(b *testing.B) {
		zl := zerolog.New(io.Discard)
		logger := log.NewCustomLogger(zl)
		zerolog := logger.Impl().(*zerolog.Logger)
		for i := 0; i < b.N; i++ {
			zerolog.Info().Int64("foo", 100000).Msg(message)
			zerolog.Info().Str("foo", "foo").Msg(message)
			zerolog.Error().
				Int64("foo", 100000).
				Str("bar", "foo").
				Bytes("other", byteSliceToLog).
				Err(errorToLog).
				Msg(message)
		}
	})

	b.Run("logger", func(b *testing.B) {
		zl := zerolog.New(io.Discard)
		logger := log.NewCustomLogger(zl)
		for i := 0; i < b.N; i++ {
			logger.Info(message, "foo", 100000)
			logger.Info(message, "foo", "foo")
			logger.Error(message, "foo", 100000, "bar", "foo", "other", byteSliceToLog, "error", errorToLog)
		}
	})
}
