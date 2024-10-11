// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"bytes"
	"fmt"
	"log"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestStdlogAdapter_PickLevel(t *testing.T) {
	t.Run("picks debug level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[DEBUG] coffee?")

		assert.Equal(t, Debug, level)
		assert.Equal(t, "coffee?", rest)
	})

	t.Run("picks trace level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[TRACE] coffee?")

		assert.Equal(t, Trace, level)
		assert.Equal(t, "coffee?", rest)
	})

	t.Run("picks info level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[INFO] coffee?")

		assert.Equal(t, Info, level)
		assert.Equal(t, "coffee?", rest)
	})

	t.Run("picks warn level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[WARN] coffee?")

		assert.Equal(t, Warn, level)
		assert.Equal(t, "coffee?", rest)
	})

	t.Run("picks error level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[ERROR] coffee?")

		assert.Equal(t, Error, level)
		assert.Equal(t, "coffee?", rest)
	})

	t.Run("picks error as err level", func(t *testing.T) {
		var s stdlogAdapter

		level, rest := s.pickLevel("[ERR] coffee?")

		assert.Equal(t, Error, level)
		assert.Equal(t, "coffee?", rest)
	})
}

func TestStdlogAdapter_TrimTimestamp(t *testing.T) {
	cases := []struct {
		name   string
		input  string
		expect string
	}{
		{
			name:   "Go log Ldate",
			input:  "2009/01/23 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "Go log Ldate|Ltime",
			input:  "2009/01/23 01:23:23 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "Go log Ldate|Ltime|Lmicroseconds",
			input:  "2009/01/23 01:23:23.123123 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "Go log Ltime",
			input:  "01:23:23 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "Go log Ltime|Lmicroseconds",
			input:  "01:23:23.123123 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "ISO 8601 date",
			input:  "2021-10-28 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "ISO 8601 date and time",
			input:  "2021-10-28T19:27:28+00:00 [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "ISO 8601 date and time zulu",
			input:  "2021-10-28T19:27:28Z [ERR] message",
			expect: "[ERR] message",
		},
		{
			name:   "no timestamp",
			input:  "[ERR] message",
			expect: "[ERR] message",
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			var s stdlogAdapter
			got := s.trimTimestamp(c.input)
			assert.Equal(t, c.expect, got)
		})
	}
}

func TestStdlogAdapter_ForceLevel(t *testing.T) {
	cases := []struct {
		name        string
		forceLevel  Level
		inferLevels bool
		write       string
		expect      string
	}{
		{
			name:       "force error",
			forceLevel: Error,
			write:      "this is a test",
			expect:     "[ERROR] test: this is a test\n",
		},
		{
			name:        "force error overrides infer",
			forceLevel:  Error,
			inferLevels: true,
			write:       "[DEBUG] this is a test",
			expect:      "[ERROR] test: this is a test\n",
		},
		{
			name:       "force error and strip debug",
			forceLevel: Error,
			write:      "[DEBUG] this is a test",
			expect:     "[ERROR] test: this is a test\n",
		},
		{
			name:       "force trace",
			forceLevel: Trace,
			write:      "this is a test",
			expect:     "[TRACE] test: this is a test\n",
		},
		{
			name:       "force trace and strip higher level error",
			forceLevel: Trace,
			write:      "[WARN] this is a test",
			expect:     "[TRACE] test: this is a test\n",
		},
		{
			name:       "force with invalid level",
			forceLevel: -10,
			write:      "this is a test",
			expect:     "[INFO]  test: this is a test\n",
		},
		{
			name:        "infer debug",
			forceLevel:  NoLevel,
			inferLevels: true,
			write:       "[DEBUG] debug info",
			expect:      "[DEBUG] test: debug info\n",
		},
		{
			name:        "info is used if not forced and cannot infer",
			forceLevel:  NoLevel,
			inferLevels: false,
			write:       "some message",
			expect:      "[INFO]  test: some message\n",
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			var stderr bytes.Buffer

			logger := New(&LoggerOptions{
				Name:   "test",
				Output: &stderr,
				Level:  Trace,
			})

			s := &stdlogAdapter{
				log:         logger,
				forceLevel:  c.forceLevel,
				inferLevels: c.inferLevels,
			}

			_, err := s.Write([]byte(c.write))
			assert.NoError(t, err)

			errStr := stderr.String()
			errDataIdx := strings.IndexByte(errStr, ' ')
			errRest := errStr[errDataIdx+1:]

			assert.Equal(t, c.expect, errRest)
		})
	}
}

func TestFromStandardLogger(t *testing.T) {
	var buf bytes.Buffer

	sl := log.New(&buf, "test-stdlib-log ", log.Ltime)

	hl := FromStandardLogger(sl, &LoggerOptions{
		Name:            "hclog-inner",
		IncludeLocation: true,
	})

	hl.Info("this is a test", "name", "tester", "count", 1)
	_, file, line, ok := runtime.Caller(0)
	require.True(t, ok)

	actual := buf.String()
	suffix := fmt.Sprintf(
		"[INFO]  go-hclog/%s:%d: hclog-inner: this is a test: name=tester count=1\n",
		filepath.Base(file), line-1)
	require.Equal(t, suffix, actual[25:])

	prefix := "test-stdlib-log "
	require.Equal(t, prefix, actual[:16])
}

func TestFromStandardLogger_helper(t *testing.T) {
	var buf bytes.Buffer

	sl := log.New(&buf, "test-stdlib-log ", log.Ltime)

	hl := FromStandardLogger(sl, &LoggerOptions{
		Name:                     "hclog-inner",
		IncludeLocation:          true,
		AdditionalLocationOffset: 1,
	})

	helper := func() {
		hl.Info("this is a test", "name", "tester", "count", 1)
	}

	helper()
	_, file, line, ok := runtime.Caller(0)
	require.True(t, ok)

	actual := buf.String()
	suffix := fmt.Sprintf(
		"[INFO]  go-hclog/%s:%d: hclog-inner: this is a test: name=tester count=1\n",
		filepath.Base(file), line-1)
	require.Equal(t, suffix, actual[25:])

	prefix := "test-stdlib-log "
	require.Equal(t, prefix, actual[:16])
}
