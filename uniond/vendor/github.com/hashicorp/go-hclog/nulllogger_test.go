// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

var logger = NewNullLogger()

func TestNullLoggerIsEfficient(t *testing.T) {
	// Since statements like "IsWarn()", "IsError()", etc. are used to gate
	// actually writing warning and error statements, the null logger will
	// be faster and more efficient if it always returns false for these calls.
	assert.False(t, logger.IsTrace())
	assert.False(t, logger.IsDebug())
	assert.False(t, logger.IsInfo())
	assert.False(t, logger.IsWarn())
	assert.False(t, logger.IsError())
}

func TestNullLoggerReturnsNullLoggers(t *testing.T) {

	// Sometimes the logger is asked to return subloggers.
	// These should also be a nullLogger.

	subLogger := logger.With()
	_, ok := subLogger.(*nullLogger)
	assert.True(t, ok)

	subLogger = logger.Named("")
	_, ok = subLogger.(*nullLogger)
	assert.True(t, ok)

	subLogger = logger.ResetNamed("")
	_, ok = subLogger.(*nullLogger)
	assert.True(t, ok)
}

func TestStandardLoggerIsntNil(t *testing.T) {
	// Don't return a nil pointer for the standard logger,
	// lest it cause a panic.
	stdLogger := logger.StandardLogger(nil)
	assert.NotEqual(t, nil, stdLogger)
}
