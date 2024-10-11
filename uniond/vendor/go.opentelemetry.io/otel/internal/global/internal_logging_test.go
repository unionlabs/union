// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"bytes"
	"errors"
	"io"
	"log"
	"sync"
	"testing"

	"github.com/go-logr/logr"

	"github.com/stretchr/testify/assert"

	"github.com/go-logr/logr/funcr"
	"github.com/go-logr/stdr"
)

func TestLoggerConcurrentSafe(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		SetLogger(stdr.New(log.New(io.Discard, "", 0)))
	}()
	wg.Add(1)
	go func() {
		defer wg.Done()
		Info("")
	}()

	wg.Wait()
	ResetForTest(t)
}

func TestLogLevel(t *testing.T) {
	tests := []struct {
		name      string
		verbosity int
		logF      func()
		want      string
	}{
		{
			name:      "Verbosity 0 should log errors.",
			verbosity: 0,
			want:      `"msg"="foobar" "error"="foobar"`,
			logF: func() {
				Error(errors.New("foobar"), "foobar")
			},
		},
		{
			name:      "Verbosity 1 should log warnings",
			verbosity: 1,
			want:      `"level"=1 "msg"="foo"`,
			logF: func() {
				Warn("foo")
			},
		},
		{
			name:      "Verbosity 4 should log info",
			verbosity: 4,
			want:      `"level"=4 "msg"="bar"`,
			logF: func() {
				Info("bar")
			},
		},
		{
			name:      "Verbosity 8 should log debug",
			verbosity: 8,
			want:      `"level"=8 "msg"="baz"`,
			logF: func() {
				Debug("baz")
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			var buf bytes.Buffer
			SetLogger(newBuffLogger(&buf, test.verbosity))

			test.logF()

			assert.Equal(t, test.want, buf.String())
		})
	}
}

func newBuffLogger(buf *bytes.Buffer, verbosity int) logr.Logger {
	return funcr.New(func(prefix, args string) {
		_, _ = buf.Write([]byte(args))
	}, funcr.Options{
		Verbosity: verbosity,
	})
}
