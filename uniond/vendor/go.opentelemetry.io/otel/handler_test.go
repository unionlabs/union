// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otel

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

type testErrHandler struct {
	err error
}

var _ ErrorHandler = &testErrHandler{}

func (eh *testErrHandler) Handle(err error) { eh.err = err }

func TestGlobalErrorHandler(t *testing.T) {
	SetErrorHandler(GetErrorHandler())
	assert.NotPanics(t, func() { Handle(assert.AnError) }, "Default assignment")

	e1 := &testErrHandler{}
	SetErrorHandler(e1)
	Handle(assert.AnError)
	assert.ErrorIs(t, e1.err, assert.AnError)
	e1.err = nil

	e2 := &testErrHandler{}
	SetErrorHandler(e2)
	GetErrorHandler().Handle(assert.AnError)
	assert.ErrorIs(t, e2.err, assert.AnError)
}
