package icmd

import (
	"bytes"
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"
	"time"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/internal/maint"
)

var (
	bindir   = fs.NewDir(maint.T, "icmd-dir")
	binname  = bindir.Join("bin-stub") + pathext()
	stubpath = filepath.FromSlash("./internal/stub")
)

func pathext() string {
	if runtime.GOOS == "windows" {
		return ".exe"
	}
	return ""
}

func TestMain(m *testing.M) {
	exitcode := m.Run()
	bindir.Remove()
	os.Exit(exitcode)
}

func buildStub(t assert.TestingT) {
	if _, err := os.Stat(binname); err == nil {
		return
	}
	result := RunCommand("go", "build", "-o", binname, stubpath)
	result.Assert(t, Success)
}

func TestRunCommandSuccess(t *testing.T) {
	buildStub(t)

	result := RunCommand(binname)
	result.Assert(t, Success)
}

func TestRunCommandWithCombined(t *testing.T) {
	buildStub(t)

	result := RunCommand(binname, "-warn")
	result.Assert(t, Expected{})

	assert.Equal(t, result.Combined(), "this is stdout\nthis is stderr\n")
	assert.Equal(t, result.Stdout(), "this is stdout\n")
	assert.Equal(t, result.Stderr(), "this is stderr\n")
}

func TestRunCommandWithTimeoutFinished(t *testing.T) {
	buildStub(t)

	result := RunCmd(Cmd{
		Command: []string{binname, "-sleep=1ms"},
		Timeout: 2 * time.Second,
	})
	result.Assert(t, Expected{Out: "this is stdout"})
}

func TestRunCommandWithTimeoutKilled(t *testing.T) {
	buildStub(t)

	command := []string{binname, "-sleep=200ms"}
	result := RunCmd(Cmd{Command: command, Timeout: 30 * time.Millisecond})
	result.Assert(t, Expected{Timeout: true, Out: None, Err: None})
}

func TestRunCommandWithErrors(t *testing.T) {
	buildStub(t)

	result := RunCommand("doesnotexists")
	expected := `exec: "doesnotexists": executable file not found`
	result.Assert(t, Expected{Out: None, Err: None, ExitCode: 127, Error: expected})
}

func TestRunCommandWithStdoutNoStderr(t *testing.T) {
	buildStub(t)

	result := RunCommand(binname)
	result.Assert(t, Expected{Out: "this is stdout\n", Err: None})
}

func TestRunCommandWithExitCode(t *testing.T) {
	buildStub(t)

	result := RunCommand(binname, "-fail=99")
	result.Assert(t, Expected{
		ExitCode: 99,
		Error:    "exit status 99",
	})
}

func TestResult_Match_NotMatched(t *testing.T) {
	result := &Result{
		Cmd:       exec.Command("binary", "arg1"),
		ExitCode:  99,
		Error:     errors.New("exit code 99"),
		outBuffer: newLockedBuffer("the output"),
		errBuffer: newLockedBuffer("the stderr"),
		Timeout:   true,
	}
	exp := Expected{
		ExitCode: 101,
		Out:      "Something else",
		Err:      None,
	}
	err := result.match(exp)
	assert.ErrorContains(t, err, "Failures")
	assert.Equal(t, err.Error(), expectedMatch)
}

var expectedMatch = `
Command:  binary arg1
ExitCode: 99 (timeout)
Error:    exit code 99
Stdout:   the output
Stderr:   the stderr

Failures:
ExitCode was 99 expected 101
Expected command to finish, but it hit the timeout
Expected stdout to contain "Something else"
Expected stderr to contain "[NOTHING]"`

func newLockedBuffer(s string) *lockedBuffer {
	return &lockedBuffer{buf: *bytes.NewBufferString(s)}
}

func TestResult_Match_NotMatchedNoError(t *testing.T) {
	result := &Result{
		Cmd:       exec.Command("binary", "arg1"),
		outBuffer: newLockedBuffer("the output"),
		errBuffer: newLockedBuffer("the stderr"),
	}
	exp := Expected{
		ExitCode: 101,
		Out:      "Something else",
		Err:      None,
	}
	err := result.match(exp)
	assert.ErrorContains(t, err, "Failures")
	assert.Equal(t, err.Error(), expectedResultMatchNoMatch)
}

var expectedResultMatchNoMatch = `
Command:  binary arg1
ExitCode: 0
Stdout:   the output
Stderr:   the stderr

Failures:
ExitCode was 0 expected 101
Expected stdout to contain "Something else"
Expected stderr to contain "[NOTHING]"`

func TestResult_Match_Match(t *testing.T) {
	result := &Result{
		Cmd:       exec.Command("binary", "arg1"),
		outBuffer: newLockedBuffer("the output"),
		errBuffer: newLockedBuffer("the stderr"),
	}
	exp := Expected{
		Out: "the output",
		Err: "the stderr",
	}
	err := result.match(exp)
	assert.NilError(t, err)
}
