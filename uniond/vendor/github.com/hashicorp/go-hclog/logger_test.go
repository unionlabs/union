// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"runtime"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type bufferingBuffer struct {
	held    bytes.Buffer
	flushed bytes.Buffer
}

func (b *bufferingBuffer) Write(p []byte) (int, error) {
	return b.held.Write(p)
}

func (b *bufferingBuffer) String() string {
	return b.flushed.String()
}

func (b *bufferingBuffer) Flush() error {
	_, err := b.flushed.WriteString(b.held.String())
	return err
}

func TestLogger(t *testing.T) {
	t.Run("uses default output if none is given", func(t *testing.T) {
		var buf bytes.Buffer
		DefaultOutput = &buf

		logger := New(&LoggerOptions{
			Name: "test",
		})

		logger.Info("this is test", "who", "programmer", "why", "testing")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=testing\n", rest)
	})

	t.Run("formats log entries", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", "testing")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=testing\n", rest)
	})

	t.Run("renders slice values specially", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", []interface{}{"testing", "dev", 1, uint64(5), []int{3, 4}})

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=[testing, dev, 1, 5, \"[3 4]\"]\n", rest)
	})

	t.Run("renders values in slices with quotes", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", []string{"testing & qa", "dev"})

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=[\"testing & qa\", \"dev\"]\n", rest)
	})

	t.Run("escapes quotes in values", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", `this is "quoted"`)

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, `[INFO]  test: this is test: who=programmer why="this is \"quoted\""`+"\n", rest)
	})

	t.Run("prints empty double quotes for empty strings", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", ``)

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, `[INFO]  test: this is test: who=programmer why=""`+"\n", rest)
	})

	t.Run("quotes when there are nonprintable sequences in a value", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", "\U0001F603")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=\"\U0001F603\"\n", rest)
	})

	t.Run("formats multiline values nicely", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", "testing\nand other\npretty cool things")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		expected := `[INFO]  test: this is test: who=programmer
  why=
  | testing
  | and other
  | pretty cool things` + "\n  \n"
		assert.Equal(t, expected, rest)
	})

	t.Run("handles backslash r in entries", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "who", "programmer", "why", "testing\n\rand other\n\rpretty cool things like \x01 and \u1680 and \U00101120")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		expected := `[INFO]  test: this is test: who=programmer
  why=
  | testing
  | \rand other
  | \rpretty cool things like \x01 and \u1680 and \U00101120` + "\n  \n"
		assert.Equal(t, expected, rest)
	})

	t.Run("outputs stack traces", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("who", "programmer", "why", "testing", Stacktrace())

		lines := strings.Split(buf.String(), "\n")
		require.True(t, len(lines) > 1)

		assert.Equal(t, "github.com/hashicorp/go-hclog.Stacktrace", lines[1])
	})

	t.Run("outputs stack traces with it's given a name", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("who", "programmer", "why", "testing", "foo", Stacktrace())

		lines := strings.Split(buf.String(), "\n")
		require.True(t, len(lines) > 1)

		assert.Equal(t, "github.com/hashicorp/go-hclog.Stacktrace", lines[1])
	})

	t.Run("prefixes the name", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			// No name!
			Output: &buf,
		})

		logger.Info("this is test")
		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]
		assert.Equal(t, "[INFO]  this is test\n", rest)

		buf.Reset()

		another := logger.Named("sublogger")
		another.Info("this is test")
		str = buf.String()
		dataIdx = strings.IndexByte(str, ' ')
		rest = str[dataIdx+1:]
		assert.Equal(t, "[INFO]  sublogger: this is test\n", rest)
	})

	t.Run("can force colors to on in any context", func(t *testing.T) {
		if runtime.GOOS == "windows" {
			t.Skip("colors are different on windows")
		}

		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			// No name!
			Output:     &buf,
			Level:      Trace,
			Color:      ForceColor,
			TimeFormat: "<time>",
		})

		logger.Trace("trace")
		logger.Debug("debug")
		logger.Info("info")
		logger.Warn("warn")
		logger.Error("error")
		str := buf.String()

		assert.Equal(t, ""+
			"\033[92m<time> [TRACE] trace\n\033[0m"+
			"\033[97m<time> [DEBUG] debug\n\033[0m"+
			"\033[94m<time> [INFO]  info\n\033[0m"+
			"\033[93m<time> [WARN]  warn\n\033[0m"+
			"\033[91m<time> [ERROR] error\n\033[0m",
			str,
		)
	})

	t.Run("use a different time format", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			TimeFormat: time.Kitchen,
		})

		logger.Info("this is test", "who", "programmer", "why", "testing is fun")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')

		assert.Equal(t, str[:dataIdx], time.Now().Format(time.Kitchen))
	})

	t.Run("use UTC time zone", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			TimeFormat: time.Kitchen,
			TimeFn:     func() time.Time { return time.Now().UTC() },
		})

		logger.Info("this is test", "who", "programmer", "why", "testing is fun")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')

		assert.Equal(t, str[:dataIdx], time.Now().UTC().Format(time.Kitchen))
	})

	t.Run("respects DisableTime", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:        "test",
			Output:      &buf,
			DisableTime: true,
		})

		logger.Info("Señorita banana")

		str := buf.String()

		assert.Equal(t, "[INFO]  test: Señorita banana\n", str)
	})

	t.Run("use with", func(t *testing.T) {
		var buf bytes.Buffer

		rootLogger := New(&LoggerOptions{
			Name:   "with_test",
			Output: &buf,
		})

		// Build the root logger in two steps, which triggers a slice capacity increase
		// and is part of the test for inadvertant slice aliasing.
		rootLogger = rootLogger.With("a", 1, "b", 2)
		rootLogger = rootLogger.With("c", 3)

		// Derive two new loggers which should be completely independent
		derived1 := rootLogger.With("cat", 30)
		derived2 := rootLogger.With("dog", 40)

		derived1.Info("test1")
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: test1: a=1 b=2 c=3 cat=30\n", output[dataIdx+1:])

		buf.Reset()

		derived2.Info("test2")
		output = buf.String()
		dataIdx = strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: test2: a=1 b=2 c=3 dog=40\n", output[dataIdx+1:])
	})

	t.Run("unpaired with", func(t *testing.T) {
		var buf bytes.Buffer

		rootLogger := New(&LoggerOptions{
			Name:   "with_test",
			Output: &buf,
		})

		derived1 := rootLogger.With("a")
		derived1.Info("test1")
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: test1: EXTRA_VALUE_AT_END=a\n", output[dataIdx+1:])
	})

	t.Run("use with and log", func(t *testing.T) {
		var buf bytes.Buffer

		rootLogger := New(&LoggerOptions{
			Name:   "with_test",
			Output: &buf,
		})

		// Build the root logger in two steps, which triggers a slice capacity increase
		// and is part of the test for inadvertant slice aliasing.
		rootLogger = rootLogger.With("a", 1, "b", 2)
		// This line is here to test that when calling With with the same key,
		// only the last value remains (see issue #21)
		rootLogger = rootLogger.With("c", 4)
		rootLogger = rootLogger.With("c", 3)

		// Derive another logger which should be completely independent of rootLogger
		derived := rootLogger.With("cat", 30)

		rootLogger.Info("root_test", "bird", 10)
		output := buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: root_test: a=1 b=2 c=3 bird=10\n", output[dataIdx+1:])

		buf.Reset()

		derived.Info("derived_test")
		output = buf.String()
		dataIdx = strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: derived_test: a=1 b=2 c=3 cat=30\n", output[dataIdx+1:])
	})

	t.Run("use with and log and change levels", func(t *testing.T) {
		var buf bytes.Buffer

		rootLogger := New(&LoggerOptions{
			Name:   "with_test",
			Output: &buf,
			Level:  Warn,
		})

		// Build the root logger in two steps, which triggers a slice capacity increase
		// and is part of the test for inadvertant slice aliasing.
		rootLogger = rootLogger.With("a", 1, "b", 2)
		rootLogger = rootLogger.With("c", 3)

		// Derive another logger which should be completely independent of rootLogger
		derived := rootLogger.With("cat", 30)

		rootLogger.Info("root_test", "bird", 10)
		output := buf.String()
		if output != "" {
			t.Fatalf("unexpected output: %s", output)
		}

		buf.Reset()

		derived.Info("derived_test")
		output = buf.String()
		if output != "" {
			t.Fatalf("unexpected output: %s", output)
		}

		derived.SetLevel(Info)

		rootLogger.Info("root_test", "bird", 10)
		output = buf.String()
		dataIdx := strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: root_test: a=1 b=2 c=3 bird=10\n", output[dataIdx+1:])

		buf.Reset()

		derived.Info("derived_test")
		output = buf.String()
		dataIdx = strings.IndexByte(output, ' ')
		assert.Equal(t, "[INFO]  with_test: derived_test: a=1 b=2 c=3 cat=30\n", output[dataIdx+1:])
	})

	t.Run("supports Printf style expansions when requested", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "production", Fmt("%d beans/day", 12))

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: production=\"12 beans/day\"\n", rest)
	})

	t.Run("supports number formating", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		logger.Info("this is test", "bytes", Hex(12), "perms", Octal(0755), "bits", Binary(5))

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: bytes=0xc perms=0755 bits=0b101\n", rest)
	})

	t.Run("supports quote formatting", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: &buf,
		})

		// unsafe is a string containing control characters and a byte
		// sequence which is invalid utf8 ("\xFFa") to assert that all
		// characters are properly encoded and produce valid utf8 output
		unsafe := "foo\nbar\bbaz\xFFa"

		logger.Info("this is test",
			"unquoted", "unquoted", "quoted", Quote("quoted"),
			"unsafeq", Quote(unsafe))

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: "+
			"unquoted=unquoted quoted=\"quoted\" "+
			"unsafeq=\"foo\\nbar\\bbaz\\xffa\"\n", rest)
	})

	t.Run("supports resetting the output", func(t *testing.T) {
		var first, second bytes.Buffer

		logger := New(&LoggerOptions{
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

		logger := New(&LoggerOptions{
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

	t.Run("named logger with disabled parent", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			// No name!
			Output:            &buf,
			Level:             Off,
			IndependentLevels: true,
		})

		logger.Info("this is test")
		str := buf.String()
		if len(str) > 0 {
			t.Fatal("output from disabled logger:", str)
		}

		buf.Reset()

		another := logger.Named("sublogger")
		another.SetLevel(Info)
		another.Info("this is test")
		str = buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]
		assert.Equal(t, "[INFO]  sublogger: this is test\n", rest)

		buf.Reset()
		logger.Info("parent should still be quiet")
		str = buf.String()
		if len(str) > 0 {
			t.Fatal("output from disabled logger:", str)
		}
	})

	t.Run("sub-loggers levels don't bubble upward", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "root",
			Output:          &buf,
			SyncParentLevel: true,
		})

		another := logger.Named("sublogger")
		another.SetLevel(Error)

		logger.Info("this is test")
		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]
		assert.Equal(t, "[INFO]  root: this is test\n", rest)

		buf.Reset()

		a := logger.Named("a")
		b := a.Named("b")
		c := a.Named("c")

		a.SetLevel(Error)

		b.Info("this is a test")

		require.Empty(t, buf.String())

		b.SetLevel(Info)

		assert.Equal(t, Error, a.GetLevel())

		a.SetLevel(Error)

		assert.Equal(t, Error, b.GetLevel())

		assert.Equal(t, Error, c.GetLevel())

		// Make sure that setting a sibling logger doesn't confuse
		// when b had previously had it's own level.
		c.SetLevel(Info)

		assert.Equal(t, Error, b.GetLevel())
	})

	t.Run("level sync example 1", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "root",
			Output:          &buf,
			SyncParentLevel: true,
		})

		s := assert.New(t)

		a := logger.Named("a")
		b := a.Named("b")
		c := a.Named("c")

		b.SetLevel(Warn)
		s.Equal(Info, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Info, c.GetLevel())

		c.SetLevel(Error)
		s.Equal(Info, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Error, c.GetLevel())

		a.SetLevel(Warn)
		s.Equal(Warn, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Warn, c.GetLevel())

		logger.SetLevel(Trace)
		s.Equal(Trace, logger.GetLevel())
		s.Equal(Trace, a.GetLevel())
		s.Equal(Trace, b.GetLevel())
		s.Equal(Trace, c.GetLevel())
	})

	t.Run("level sync example 2", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "root",
			Output:          &buf,
			SyncParentLevel: true,
		})

		s := assert.New(t)

		a := logger.Named("a")
		a.SetLevel(Error)
		b := a.Named("b")
		c := a.Named("c")
		s.Equal(Error, b.GetLevel())
		s.Equal(Error, c.GetLevel())

		b.SetLevel(Info)
		s.Equal(Error, a.GetLevel())
		s.Equal(Info, b.GetLevel())
		s.Equal(Error, c.GetLevel())

		a.SetLevel(Warn)
		s.Equal(Warn, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Warn, c.GetLevel())

		logger.SetLevel(Trace)
		s.Equal(Trace, logger.GetLevel())
		s.Equal(Trace, a.GetLevel())
		s.Equal(Trace, b.GetLevel())
		s.Equal(Trace, c.GetLevel())
	})
	t.Run("level sync example 3", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "root",
			Output:          &buf,
			SyncParentLevel: true,
		})

		s := assert.New(t)

		a := logger.Named("a")
		b := a.Named("b")

		a.SetLevel(Trace)
		s.Equal(Trace, a.GetLevel())
		s.Equal(Trace, b.GetLevel())

		b.SetLevel(Warn)
		s.Equal(Trace, a.GetLevel())
		s.Equal(Warn, b.GetLevel())

		c := a.Named("c")

		c.SetLevel(Error)
		s.Equal(Trace, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Error, c.GetLevel())

		a.SetLevel(Warn)
		s.Equal(Warn, a.GetLevel())
		s.Equal(Warn, b.GetLevel())
		s.Equal(Warn, c.GetLevel())
	})
}

func TestLogger_leveledWriter(t *testing.T) {
	t.Run("writes errors to stderr", func(t *testing.T) {
		var stderr bytes.Buffer
		var stdout bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: NewLeveledWriter(&stdout, map[Level]io.Writer{Error: &stderr}),
		})

		logger.Error("this is an error", "who", "programmer", "why", "testing")

		errStr := stderr.String()
		errDataIdx := strings.IndexByte(errStr, ' ')
		errRest := errStr[errDataIdx+1:]

		assert.Equal(t, "[ERROR] test: this is an error: who=programmer why=testing\n", errRest)
	})

	t.Run("writes non-errors to stdout", func(t *testing.T) {
		var stderr bytes.Buffer
		var stdout bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: NewLeveledWriter(&stdout, map[Level]io.Writer{Error: &stderr}),
		})

		logger.Info("this is test", "who", "programmer", "why", "testing")

		outStr := stdout.String()
		outDataIdx := strings.IndexByte(outStr, ' ')
		outRest := outStr[outDataIdx+1:]

		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=testing\n", outRest)
	})

	t.Run("writes errors and non-errors correctly", func(t *testing.T) {
		var stderr bytes.Buffer
		var stdout bytes.Buffer

		logger := New(&LoggerOptions{
			Name:   "test",
			Output: NewLeveledWriter(&stdout, map[Level]io.Writer{Error: &stderr}),
		})

		logger.Info("this is test", "who", "programmer", "why", "testing")
		logger.Error("this is an error", "who", "programmer", "why", "testing")

		errStr := stderr.String()
		errDataIdx := strings.IndexByte(errStr, ' ')
		errRest := errStr[errDataIdx+1:]

		outStr := stdout.String()
		outDataIdx := strings.IndexByte(outStr, ' ')
		outRest := outStr[outDataIdx+1:]

		assert.Equal(t, "[ERROR] test: this is an error: who=programmer why=testing\n", errRest)
		assert.Equal(t, "[INFO]  test: this is test: who=programmer why=testing\n", outRest)
	})
}

func TestLogger_JSON(t *testing.T) {
	t.Run("json formatting", func(t *testing.T) {
		var buf bytes.Buffer
		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		logger.Info("this is test", "who", "programmer", "why", "testing is fun")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "programmer", raw["who"])
		assert.Equal(t, "testing is fun", raw["why"])
	})

	t.Run("use a different time format", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
			TimeFormat: time.Kitchen,
		})

		logger.Info("Lacatan banana")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		val, ok := raw["@timestamp"]
		if !ok {
			t.Fatal("missing '@timestamp' key")
		}

		assert.Equal(t, val, time.Now().Format(time.Kitchen))
	})

	t.Run("use UTC time zone", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
			TimeFormat: time.Kitchen,
			TimeFn:     func() time.Time { return time.Now().UTC() },
		})

		logger.Info("Lacatan banana")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		val, ok := raw["@timestamp"]
		if !ok {
			t.Fatal("missing '@timestamp' key")
		}

		assert.Equal(t, val, time.Now().UTC().Format(time.Kitchen))
	})

	t.Run("respects DisableTime", func(t *testing.T) {
		var buf bytes.Buffer
		logger := New(&LoggerOptions{
			Name:        "test",
			Output:      &buf,
			JSONFormat:  true,
			DisableTime: true,
		})

		logger.Info("Señorita banana")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		if val, ok := raw["@timestamp"]; ok {
			t.Fatalf("got: '@timestamp' key (with value %v); want: no '@timestamp' key", val)
		}
	})

	t.Run("json formatting with", func(t *testing.T) {
		var buf bytes.Buffer
		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})
		logger = logger.With("cat", "in the hat", "dog", 42)

		logger.Info("this is test", "who", "programmer", "why", "testing is fun")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "programmer", raw["who"])
		assert.Equal(t, "testing is fun", raw["why"])
		assert.Equal(t, "in the hat", raw["cat"])
		assert.Equal(t, float64(42), raw["dog"])
	})

	t.Run("json formatting error type", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		errMsg := errors.New("this is an error")
		logger.Info("this is test", "who", "programmer", "err", errMsg)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "programmer", raw["who"])
		assert.Equal(t, errMsg.Error(), raw["err"])
	})

	t.Run("json formatting custom error type json marshaler", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		errMsg := &customErrJSON{"this is an error"}
		rawMsg, err := errMsg.MarshalJSON()
		if err != nil {
			t.Fatal(err)
		}
		expectedMsg, err := strconv.Unquote(string(rawMsg))
		if err != nil {
			t.Fatal(err)
		}

		logger.Info("this is test", "who", "programmer", "err", errMsg)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "programmer", raw["who"])
		assert.Equal(t, expectedMsg, raw["err"])
	})

	t.Run("json formatting custom error type text marshaler", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		errMsg := &customErrText{"this is an error"}
		rawMsg, err := errMsg.MarshalText()
		if err != nil {
			t.Fatal(err)
		}
		expectedMsg := string(rawMsg)

		logger.Info("this is test", "who", "programmer", "err", errMsg)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "programmer", raw["who"])
		assert.Equal(t, expectedMsg, raw["err"])
	})

	t.Run("supports Printf style expansions when requested", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		logger.Info("this is test", "production", Fmt("%d beans/day", 12))

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "12 beans/day", raw["production"])
	})

	t.Run("ignores number formatting requests", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		logger.Info("this is test", "bytes", Hex(12), "perms", Octal(0755), "bits", Binary(5))

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, float64(12), raw["bytes"])
		assert.Equal(t, float64(0755), raw["perms"])
		assert.Equal(t, float64(5), raw["bits"])
	})

	t.Run("ignores quote formatting requests", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		// unsafe is a string containing control characters and a byte
		// sequence which is invalid utf8 ("\xFFa") to assert that all
		// characters are properly encoded and produce valid json
		unsafe := "foo\nbar\bbaz\xFFa"

		logger.Info("this is test",
			"unquoted", "unquoted", "quoted", Quote("quoted"),
			"unsafeq", Quote(unsafe), "unsafe", unsafe)

		b := buf.Bytes()

		// Assert the JSON only contains valid utf8 strings with the
		// illegal byte replaced with the utf8 replacement character,
		// and not invalid json with byte(255)
		// Note: testify/assert.Contains did not work here
		if needle := []byte(`\ufffda`); !bytes.Contains(b, needle) {
			t.Fatalf("could not find %q (%v) in json bytes: %q", needle, needle, b)
		}
		if needle := []byte{255, 'a'}; bytes.Contains(b, needle) {
			t.Fatalf("found %q (%v) in json bytes: %q", needle, needle, b)
		}

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, "unquoted", raw["unquoted"])
		assert.Equal(t, "quoted", raw["quoted"])
		assert.Equal(t, "foo\nbar\bbaz\uFFFDa", raw["unsafe"])
		assert.Equal(t, "foo\nbar\bbaz\uFFFDa", raw["unsafeq"])
	})

	t.Run("includes the caller location", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "test",
			Output:          &buf,
			JSONFormat:      true,
			IncludeLocation: true,
		})

		logger.Info("this is test")
		_, file, line, ok := runtime.Caller(0)
		require.True(t, ok)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, fmt.Sprintf("%v:%d", file, line-1), raw["@caller"])
	})

	t.Run("includes the caller location excluding helper functions", func(t *testing.T) {
		var buf bytes.Buffer

		logMe := func(l Logger) {
			l.Info("this is test", "who", "programmer", "why", "testing is fun")
		}

		logger := New(&LoggerOptions{
			Name:                     "test",
			Output:                   &buf,
			JSONFormat:               true,
			IncludeLocation:          true,
			AdditionalLocationOffset: 1,
		})

		logMe(logger)
		_, file, line, ok := runtime.Caller(0)
		require.True(t, ok)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, fmt.Sprintf("%v:%d", file, line-1), raw["@caller"])
	})

	t.Run("handles non-serializable entries", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:       "test",
			Output:     &buf,
			JSONFormat: true,
		})

		myfunc := func() int { return 42 }
		logger.Info("this is test", "production", myfunc)

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test", raw["@message"])
		assert.Equal(t, errJsonUnsupportedTypeMsg, raw["@warn"])
	})

	t.Run("omits the entry for the message when empty", func(t *testing.T) {
		var buf bytes.Buffer
		DefaultOutput = &buf

		logger := New(&LoggerOptions{
			Name: "test",
		})

		logger.Info("", "who", "programmer", "why", "testing")

		str := buf.String()
		dataIdx := strings.IndexByte(str, ' ')
		rest := str[dataIdx+1:]

		assert.Equal(t, "[INFO]  test: who=programmer why=testing\n", rest)
	})

	t.Run("disable json escape when log special character", func(t *testing.T) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:               "test",
			Output:             &buf,
			JSONFormat:         true,
			JSONEscapeDisabled: true,
		})

		logger.Info("this is test and use > < &")

		b := buf.Bytes()

		var raw map[string]interface{}
		if err := json.Unmarshal(b, &raw); err != nil {
			t.Fatal(err)
		}

		assert.Equal(t, "this is test and use > < &", raw["@message"])
	})

}

type customErrJSON struct {
	Message string
}

// error impl.
func (c *customErrJSON) Error() string {
	return c.Message
}

// json.Marshaler impl.
func (c customErrJSON) MarshalJSON() ([]byte, error) {
	return []byte(strconv.Quote(fmt.Sprintf("json-marshaler: %s", c.Message))), nil
}

type customErrText struct {
	Message string
}

// error impl.
func (c *customErrText) Error() string {
	return c.Message
}

// text.Marshaler impl.
func (c customErrText) MarshalText() ([]byte, error) {
	return []byte(fmt.Sprintf("text-marshaler: %s", c.Message)), nil
}

func BenchmarkLogger(b *testing.B) {
	b.Run("info with 10 pairs", func(b *testing.B) {
		var buf bytes.Buffer

		logger := New(&LoggerOptions{
			Name:            "test",
			Output:          &buf,
			IncludeLocation: true,
		})

		for i := 0; i < b.N; i++ {
			logger.Info("this is some message",
				"name", "foo",
				"what", "benchmarking yourself",
				"why", "to see what's slow",
				"k4", "value",
				"k5", "value",
				"k6", "value",
				"k7", "value",
				"k8", "value",
				"k9", "value",
				"k10", "value",
			)
		}
	})
}
