// Package cmdtest contains a framework for testing cobra Commands within Go unit tests.
package cmdtest

import (
	"bytes"
	"context"
	"io"

	"github.com/spf13/cobra"
)

// System is a system under test.
type System struct {
	commands []*cobra.Command
}

// NewSystem returns a new System.
func NewSystem() *System {
	// We aren't doing any special initialization yet,
	// but let's encourage a constructor to make it simpler
	// to update later, if needed.
	return new(System)
}

// AddCommands sets commands to be available to the Run family of methods on s.
func (s *System) AddCommands(cmds ...*cobra.Command) {
	s.commands = append(s.commands, cmds...)
}

// RunResult is the stdout and stderr resulting from a call to a System's Run family of methods,
// and any error that was returned.
type RunResult struct {
	Stdout, Stderr bytes.Buffer

	Err error
}

// Run calls s.RunC with context.Background().
func (s *System) Run(args ...string) RunResult {
	return s.RunC(context.Background(), args...)
}

// RunC calls s.RunWithInput with an empty stdin.
func (s *System) RunC(ctx context.Context, args ...string) RunResult {
	return s.RunWithInputC(ctx, bytes.NewReader(nil), args...)
}

// RunWithInput calls s.RunWithInputC with context.Background().
func (s *System) RunWithInput(in io.Reader, args ...string) RunResult {
	return s.RunWithInputC(context.Background(), in, args...)
}

// RunWithInputC executes a new root command with subcommands
// that were set in s.AddCommands().
// The command's stdin is set to the in argument.
// RunWithInputC returns a RunResult wrapping stdout, stderr, and any returned error.
func (s *System) RunWithInputC(ctx context.Context, in io.Reader, args ...string) RunResult {
	rootCmd := &cobra.Command{}
	rootCmd.AddCommand(s.commands...)

	rootCmd.SetIn(in)

	var res RunResult
	rootCmd.SetOutput(&res.Stdout)
	rootCmd.SetErr(&res.Stderr)

	rootCmd.SetArgs(args)

	res.Err = rootCmd.ExecuteContext(ctx)
	return res
}

// MustRun calls s.Run, but also calls t.FailNow if RunResult.Err is not nil.
func (s *System) MustRun(t TestingT, args ...string) RunResult {
	t.Helper()

	return s.MustRunC(t, context.Background(), args...)
}

// MustRunC calls s.RunWithInput, but also calls t.FailNow if RunResult.Err is not nil.
func (s *System) MustRunC(t TestingT, ctx context.Context, args ...string) RunResult {
	t.Helper()

	return s.MustRunWithInputC(t, ctx, bytes.NewReader(nil), args...)
}

// MustRunWithInput calls s.RunWithInput, but also calls t.FailNow if RunResult.Err is not nil.
func (s *System) MustRunWithInput(t TestingT, in io.Reader, args ...string) RunResult {
	t.Helper()

	return s.MustRunWithInputC(t, context.Background(), in, args...)
}

// MustRunWithInputC calls s.RunWithInputC, but also calls t.FailNow if RunResult.Err is not nil.
func (s *System) MustRunWithInputC(t TestingT, ctx context.Context, in io.Reader, args ...string) RunResult {
	t.Helper()

	res := s.RunWithInputC(ctx, in, args...)
	if res.Err != nil {
		t.Logf("Error executing %v: %v", args, res.Err)
		t.Logf("Stdout: %q", res.Stdout.String())
		t.Logf("Stderr: %q", res.Stderr.String())
		t.FailNow()
	}

	return res
}

// TestingT is a subset of testing.TB,
// containing only what the (*System).Must methods use.
//
// This simplifies using other testing wrappers,
// such as testify suite, etc.
type TestingT interface {
	Helper()

	Logf(format string, args ...any)

	FailNow()
}
