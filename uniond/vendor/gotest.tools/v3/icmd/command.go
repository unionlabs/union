/*Package icmd executes binaries and provides convenient assertions for testing the results.
 */
package icmd // import "gotest.tools/v3/icmd"

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"sync"
	"time"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

type helperT interface {
	Helper()
}

// None is a token to inform Result.Assert that the output should be empty
const None = "[NOTHING]"

type lockedBuffer struct {
	m   sync.RWMutex
	buf bytes.Buffer
}

func (buf *lockedBuffer) Write(b []byte) (int, error) {
	buf.m.Lock()
	defer buf.m.Unlock()
	return buf.buf.Write(b)
}

func (buf *lockedBuffer) String() string {
	buf.m.RLock()
	defer buf.m.RUnlock()
	return buf.buf.String()
}

// Result stores the result of running a command
type Result struct {
	Cmd      *exec.Cmd
	ExitCode int
	Error    error
	// Timeout is true if the command was killed because it ran for too long
	Timeout   bool
	outBuffer *lockedBuffer
	errBuffer *lockedBuffer
}

// Assert compares the Result against the Expected struct, and fails the test if
// any of the expectations are not met.
//
// This function is equivalent to assert.Assert(t, result.Equal(exp)).
func (r *Result) Assert(t assert.TestingT, exp Expected) *Result {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	assert.Assert(t, r.Equal(exp))
	return r
}

// Equal compares the result to Expected. If the result doesn't match expected
// returns a formatted failure message with the command, stdout, stderr, exit code,
// and any failed expectations.
func (r *Result) Equal(exp Expected) cmp.Comparison {
	return func() cmp.Result {
		return cmp.ResultFromError(r.match(exp))
	}
}

// Compare the result to Expected and return an error if they do not match.
func (r *Result) Compare(exp Expected) error {
	return r.match(exp)
}

func (r *Result) match(exp Expected) error {
	errors := []string{}
	add := func(format string, args ...interface{}) {
		errors = append(errors, fmt.Sprintf(format, args...))
	}

	if exp.ExitCode != r.ExitCode {
		add("ExitCode was %d expected %d", r.ExitCode, exp.ExitCode)
	}
	if exp.Timeout != r.Timeout {
		if exp.Timeout {
			add("Expected command to timeout")
		} else {
			add("Expected command to finish, but it hit the timeout")
		}
	}
	if !matchOutput(exp.Out, r.Stdout()) {
		add("Expected stdout to contain %q", exp.Out)
	}
	if !matchOutput(exp.Err, r.Stderr()) {
		add("Expected stderr to contain %q", exp.Err)
	}
	switch {
	// If a non-zero exit code is expected there is going to be an error.
	// Don't require an error message as well as an exit code because the
	// error message is going to be "exit status <code> which is not useful
	case exp.Error == "" && exp.ExitCode != 0:
	case exp.Error == "" && r.Error != nil:
		add("Expected no error")
	case exp.Error != "" && r.Error == nil:
		add("Expected error to contain %q, but there was no error", exp.Error)
	case exp.Error != "" && !strings.Contains(r.Error.Error(), exp.Error):
		add("Expected error to contain %q", exp.Error)
	}

	if len(errors) == 0 {
		return nil
	}
	return fmt.Errorf("%s\nFailures:\n%s", r, strings.Join(errors, "\n"))
}

func matchOutput(expected string, actual string) bool {
	switch expected {
	case None:
		return actual == ""
	default:
		return strings.Contains(actual, expected)
	}
}

func (r *Result) String() string {
	var timeout string
	if r.Timeout {
		timeout = " (timeout)"
	}
	var errString string
	if r.Error != nil {
		errString = "\nError:    " + r.Error.Error()
	}

	return fmt.Sprintf(`
Command:  %s
ExitCode: %d%s%s
Stdout:   %v
Stderr:   %v
`,
		strings.Join(r.Cmd.Args, " "),
		r.ExitCode,
		timeout,
		errString,
		r.Stdout(),
		r.Stderr())
}

// Expected is the expected output from a Command. This struct is compared to a
// Result struct by Result.Assert().
type Expected struct {
	ExitCode int
	Timeout  bool
	Error    string
	Out      string
	Err      string
}

// Success is the default expected result. A Success result is one with a 0
// ExitCode.
var Success = Expected{}

// Stdout returns the stdout of the process as a string
func (r *Result) Stdout() string {
	return r.outBuffer.String()
}

// Stderr returns the stderr of the process as a string
func (r *Result) Stderr() string {
	return r.errBuffer.String()
}

// Combined returns the stdout and stderr combined into a single string
func (r *Result) Combined() string {
	return r.outBuffer.String() + r.errBuffer.String()
}

func (r *Result) setExitError(err error) {
	if err == nil {
		return
	}
	r.Error = err
	r.ExitCode = processExitCode(err)
}

// Cmd contains the arguments and options for a process to run as part of a test
// suite.
type Cmd struct {
	Command    []string
	Timeout    time.Duration
	Stdin      io.Reader
	Stdout     io.Writer
	Stderr     io.Writer
	Dir        string
	Env        []string
	ExtraFiles []*os.File
}

// Command create a simple Cmd with the specified command and arguments
func Command(command string, args ...string) Cmd {
	return Cmd{Command: append([]string{command}, args...)}
}

// RunCmd runs a command and returns a Result
func RunCmd(cmd Cmd, cmdOperators ...CmdOp) *Result {
	result := StartCmd(cmd, cmdOperators...)
	if result.Error != nil {
		return result
	}
	return WaitOnCmd(cmd.Timeout, result)
}

// RunCommand runs a command with default options, and returns a result
func RunCommand(command string, args ...string) *Result {
	return RunCmd(Command(command, args...))
}

// StartCmd starts a command, but doesn't wait for it to finish
func StartCmd(cmd Cmd, cmdOperators ...CmdOp) *Result {
	for _, op := range cmdOperators {
		op(&cmd)
	}
	result := buildCmd(cmd)
	if result.Error != nil {
		return result
	}
	result.setExitError(result.Cmd.Start())
	return result
}

// TODO: support exec.CommandContext
func buildCmd(cmd Cmd) *Result {
	var execCmd *exec.Cmd
	switch len(cmd.Command) {
	case 1:
		execCmd = exec.Command(cmd.Command[0])
	default:
		execCmd = exec.Command(cmd.Command[0], cmd.Command[1:]...)
	}
	outBuffer := new(lockedBuffer)
	errBuffer := new(lockedBuffer)

	execCmd.Stdin = cmd.Stdin
	execCmd.Dir = cmd.Dir
	execCmd.Env = cmd.Env
	if cmd.Stdout != nil {
		execCmd.Stdout = io.MultiWriter(outBuffer, cmd.Stdout)
	} else {
		execCmd.Stdout = outBuffer
	}
	if cmd.Stderr != nil {
		execCmd.Stderr = io.MultiWriter(errBuffer, cmd.Stderr)
	} else {
		execCmd.Stderr = errBuffer
	}
	execCmd.ExtraFiles = cmd.ExtraFiles

	return &Result{
		Cmd:       execCmd,
		outBuffer: outBuffer,
		errBuffer: errBuffer,
	}
}

// WaitOnCmd waits for a command to complete. If timeout is non-nil then
// only wait until the timeout.
func WaitOnCmd(timeout time.Duration, result *Result) *Result {
	if timeout == time.Duration(0) {
		result.setExitError(result.Cmd.Wait())
		return result
	}

	done := make(chan error, 1)
	// Wait for command to exit in a goroutine
	go func() {
		done <- result.Cmd.Wait()
	}()

	select {
	case <-time.After(timeout):
		killErr := result.Cmd.Process.Kill()
		if killErr != nil {
			fmt.Printf("failed to kill (pid=%d): %v\n", result.Cmd.Process.Pid, killErr)
		}
		result.Timeout = true
	case err := <-done:
		result.setExitError(err)
	}
	return result
}
