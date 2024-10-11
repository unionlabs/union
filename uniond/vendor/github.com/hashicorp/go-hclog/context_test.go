// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"bytes"
	"context"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestContext_simpleLogger(t *testing.T) {
	l := L()
	ctx := WithContext(context.Background(), l)
	require.Equal(t, l, FromContext(ctx))
}

func TestContext_empty(t *testing.T) {
	require.Equal(t, L(), FromContext(context.Background()))
}

func TestContext_fields(t *testing.T) {
	var buf bytes.Buffer
	l := New(&LoggerOptions{
		Level:  Debug,
		Output: &buf,
	})

	// Insert the logger with fields
	ctx := WithContext(context.Background(), l, "hello", "world")
	l = FromContext(ctx)
	require.NotNil(t, l)

	// Log something so we can test the output that the field is there
	l.Debug("test")
	require.Contains(t, buf.String(), "hello")
}
