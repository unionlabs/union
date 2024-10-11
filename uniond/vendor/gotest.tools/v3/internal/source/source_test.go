package source_test

// using a separate package for test to avoid circular imports with the assert
// package

import (
	"fmt"
	"runtime"
	"strings"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/internal/source"
	"gotest.tools/v3/skip"
)

func TestFormattedCallExprArg_SingleLine(t *testing.T) {
	msg, err := shim("not", "this", "this text")
	assert.NilError(t, err)
	assert.Equal(t, `"this text"`, msg)
}

func TestFormattedCallExprArg_MultiLine(t *testing.T) {
	msg, err := shim(
		"first",
		"second",
		"this text",
	)
	assert.NilError(t, err)
	assert.Equal(t, `"this text"`, msg)
}

func TestFormattedCallExprArg_IfStatement(t *testing.T) {
	if msg, err := shim(
		"first",
		"second",
		"this text",
	); true {
		assert.NilError(t, err)
		assert.Equal(t, `"this text"`, msg)
	}
}

func shim(_, _, _ string) (string, error) {
	return source.FormattedCallExprArg(1, 2)
}

func TestFormattedCallExprArg_InDefer(t *testing.T) {
	skip.If(t, isGoVersion18)
	cap := &capture{}
	func() {
		defer cap.shim("first", "second")
	}()

	assert.NilError(t, cap.err)
	assert.Equal(t, cap.value, `"second"`)
}

func isGoVersion18() bool {
	return strings.HasPrefix(runtime.Version(), "go1.8.")
}

type capture struct {
	value string
	err   error
}

func (c *capture) shim(_, _ string) {
	c.value, c.err = source.FormattedCallExprArg(1, 1)
}

func TestFormattedCallExprArg_InAnonymousDefer(t *testing.T) {
	cap := &capture{}
	func() {
		fmt.Println()
		defer fmt.Println()
		defer func() { cap.shim("first", "second") }()
	}()

	assert.NilError(t, cap.err)
	assert.Equal(t, cap.value, `"second"`)
}

func TestFormattedCallExprArg_InDeferMultipleDefers(t *testing.T) {
	skip.If(t, isGoVersion18)
	cap := &capture{}
	func() {
		fmt.Println()
		defer fmt.Println()
		defer cap.shim("first", "second")
	}()

	assert.ErrorContains(t, cap.err, "ambiguous call expression")
}
