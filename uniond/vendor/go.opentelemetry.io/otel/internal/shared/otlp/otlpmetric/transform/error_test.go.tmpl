// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/otlpmetric/transform/error_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package transform

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var (
	e0 = errMetric{m: pbMetrics[0], err: errUnknownAggregation}
	e1 = errMetric{m: pbMetrics[1], err: errUnknownTemporality}
)

type testingErr struct{}

func (testingErr) Error() string { return "testing error" }

// errFunc is a non-comparable error type.
type errFunc func() string

func (e errFunc) Error() string {
	return e()
}

func TestMultiErr(t *testing.T) {
	const name = "TestMultiErr"
	me := &multiErr{datatype: name}

	t.Run("ErrOrNil", func(t *testing.T) {
		require.Nil(t, me.errOrNil())
		me.errs = []error{e0}
		assert.Error(t, me.errOrNil())
	})

	var testErr testingErr
	t.Run("AppendError", func(t *testing.T) {
		me.append(testErr)
		assert.Equal(t, testErr, me.errs[len(me.errs)-1])
	})

	t.Run("AppendFlattens", func(t *testing.T) {
		other := &multiErr{datatype: "OtherTestMultiErr", errs: []error{e1}}
		me.append(other)
		assert.Equal(t, e1, me.errs[len(me.errs)-1])
	})

	t.Run("ErrorMessage", func(t *testing.T) {
		// Test the overall structure of the message, but not the exact
		// language so this doesn't become a change-indicator.
		msg := me.Error()
		lines := strings.Split(msg, "\n")
		assert.Equalf(t, 4, len(lines), "expected a 4 line error message, got:\n\n%s", msg)
		assert.Contains(t, msg, name)
		assert.Contains(t, msg, e0.Error())
		assert.Contains(t, msg, testErr.Error())
		assert.Contains(t, msg, e1.Error())
	})

	t.Run("ErrorIs", func(t *testing.T) {
		assert.ErrorIs(t, me, errUnknownAggregation)
		assert.ErrorIs(t, me, e0)
		assert.ErrorIs(t, me, testErr)
		assert.ErrorIs(t, me, errUnknownTemporality)
		assert.ErrorIs(t, me, e1)

		errUnknown := errFunc(func() string { return "unknown error" })
		assert.NotErrorIs(t, me, errUnknown)

		var empty multiErr
		assert.NotErrorIs(t, &empty, errUnknownTemporality)
	})
}
