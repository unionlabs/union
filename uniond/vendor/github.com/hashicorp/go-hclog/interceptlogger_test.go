// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"bytes"
	"encoding/json"
	"fmt"
	"runtime"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInterceptLogger(t *testing.T) {
	t.Run("sends output to registered sinks", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Level:  Info,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})

		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		intercept.Debug("test log", "who", "programmer")

		str := sbuf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[DEBUG] test log: who=programmer\n", rest)

	})

	t.Run("sink includes with arguments", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_test",
			Level:  Info,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		derived := intercept.With("a", 1, "b", 2)
		derived = derived.With("c", 3)

		derived.Info("test1")
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]

		assert.Equal(t, "[INFO]  with_test: test1: a=1 b=2 c=3\n", rest)

		// Ensure intercept works
		output = sbuf.String()
		dataIdx = strings.IndexByte(output, ' ')
		rest = output[dataIdx+1:]

		assert.Equal(t, "[INFO]  with_test: test1: a=1 b=2 c=3\n", rest)
	})

	t.Run("sink includes name", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_test",
			Level:  Info,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		httpLogger := intercept.Named("http")

		httpLogger.Info("test1")
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]

		assert.Equal(t, "[INFO]  with_test.http: test1\n", rest)

		// Ensure intercept works
		output = sbuf.String()
		dataIdx = strings.IndexByte(output, ' ')
		rest = output[dataIdx+1:]

		assert.Equal(t, "[INFO]  with_test.http: test1\n", rest)
	})

	t.Run("intercepting logger can create logger with reset name", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_test",
			Level:  Info,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		httpLogger := intercept.ResetNamed("http")

		httpLogger.Info("test1")
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]

		assert.Equal(t, "[INFO]  http: test1\n", rest)

		// Ensure intercept works
		output = sbuf.String()
		dataIdx = strings.IndexByte(output, ' ')
		rest = output[dataIdx+1:]

		assert.Equal(t, "[INFO]  http: test1\n", rest)
	})

	t.Run("Intercepting logger sink can deregister itself", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_test",
			Level:  Info,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		intercept.DeregisterSink(sink)

		intercept.Info("test1")

		assert.Equal(t, "", sbuf.String())
	})

	t.Run("Sinks accept different log formats", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Level:           Info,
			Output:          &buf,
			IncludeLocation: true,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:           Debug,
			Output:          &sbuf,
			JSONFormat:      true,
			IncludeLocation: true,
		})

		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		intercept.Info("this is a test", "who", "caller")
		_, file, line, ok := runtime.Caller(0)
		require.True(t, ok)

		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]

		expected := fmt.Sprintf("[INFO]  go-hclog/interceptlogger_test.go:%d: this is a test: who=caller\n", line-1)
		assert.Equal(t, expected, rest)

		b := sbuf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is a test", raw["@message"])
		assert.Equal(t, "caller", raw["who"])
		assert.Equal(t, fmt.Sprintf("%v:%d", file, line-1), raw["@caller"])
	})

	t.Run("handles parent with arguments and log level args", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_test",
			Level:  Debug,
			Output: &buf,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		named := intercept.Named("sub_logger")
		named = named.With("parent", "logger")
		subNamed := named.Named("http")

		subNamed.Debug("test1", "path", "/some/test/path", "args", []string{"test", "test"})

		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]
		assert.Equal(t, "[DEBUG] with_test.sub_logger.http: test1: parent=logger path=/some/test/path args=[\"test\", \"test\"]\n", rest)
	})

	t.Run("derived standard loggers send output to sinks", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		intercept := NewInterceptLogger(&LoggerOptions{
			Name:   "with_name",
			Level:  Debug,
			Output: &buf,
		})

		standard := intercept.StandardLogger(&StandardLoggerOptions{InferLevels: true})

		sink := NewSinkAdapter(&LoggerOptions{
			Level:  Debug,
			Output: &sbuf,
		})
		intercept.RegisterSink(sink)
		defer intercept.DeregisterSink(sink)

		standard.Println("[DEBUG] test log")

		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		rest := output[dataIdx+1:]
		assert.Equal(t, "[DEBUG] with_name: test log\n", rest)

		output = sbuf.String()
		dataIdx = strings.IndexByte(output, ' ')
		rest = output[dataIdx+1:]
		assert.Equal(t, "[DEBUG] with_name: test log\n", rest)
	})

	t.Run("includes the caller location", func(t *testing.T) {
		var buf bytes.Buffer
		var sbuf bytes.Buffer

		logger := NewInterceptLogger(&LoggerOptions{
			Name:            "test",
			Output:          &buf,
			IncludeLocation: true,
		})

		sink := NewSinkAdapter(&LoggerOptions{
			IncludeLocation: true,
			Level:           Debug,
			Output:          &sbuf,
		})
		logger.RegisterSink(sink)
		defer logger.DeregisterSink(sink)

		logger.Info("this is test", "who", "programmer", "why", "testing is fun")
		_, _, line, ok := runtime.Caller(0)
		require.True(t, ok)

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		expected := fmt.Sprintf("[INFO]  go-hclog/interceptlogger_test.go:%d: test: this is test: who=programmer why=\"testing is fun\"\n", line-1)
		assert.Equal(t, expected, rest)

		str = sbuf.String()
		dataIdx = strings.IndexByte(str, ' ')
		rest = str[dataIdx+1:]
		assert.Equal(t, expected, rest)
	})

	t.Run("supports resetting the output", func(t *testing.T) {
		var first, second bytes.Buffer

		logger := NewInterceptLogger(&LoggerOptions{
			Output: &first,
		})

		logger.Info("this is test", "production", Fmt("%d beans/day", 12))

		str := first.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  this is test: production=\"12 beans/day\"\n", rest)

		logger.(OutputResettable).ResetOutput(&LoggerOptions{
			Output: &second,
		})

		logger.Info("this is another test", "production", Fmt("%d beans/day", 13))

		str = first.String()
		dataIdx = strings.IndexByte(str, ' ')
		rest = str[dataIdx+1:]
		assert.Equal(t, "[INFO]  this is test: production=\"12 beans/day\"\n", rest)

		str = second.String()
		dataIdx = strings.IndexByte(str, ' ')
		rest = str[dataIdx+1:]
		assert.Equal(t, "[INFO]  this is another test: production=\"13 beans/day\"\n", rest)
	})

	t.Run("supports resetting the output with flushing", func(t *testing.T) {
		var first bufferingBuffer
		var second bytes.Buffer

		logger := NewInterceptLogger(&LoggerOptions{
			Output: &first,
		})

		logger.Info("this is test", "production", Fmt("%d beans/day", 12))

		str := first.String()
		assert.Empty(t, str)

		logger.(OutputResettable).ResetOutputWithFlush(&LoggerOptions{
			Output: &second,
		}, &first)

		logger.Info("this is another test", "production", Fmt("%d beans/day", 13))

		str = first.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]
		assert.Equal(t, "[INFO]  this is test: production=\"12 beans/day\"\n", rest)

		str = second.String()
		dataIdx = strings.IndexByte(str, ' ')
		rest = str[dataIdx+1:]
		assert.Equal(t, "[INFO]  this is another test: production=\"13 beans/day\"\n", rest)
	})
}
