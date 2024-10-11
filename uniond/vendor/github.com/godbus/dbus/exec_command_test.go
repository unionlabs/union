package dbus

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"testing"
)

// How to mock exec.Command for unit tests
// https://stackoverflow.com/q/45789101/10513533

var mockedExitStatus = 0
var mockedStdout string

func fakeExecCommand(command string, args ...string) *exec.Cmd {
	cs := []string{"-test.run=TestExecCommandHelper", "--", command}
	cs = append(cs, args...)
	cmd := exec.Command(os.Args[0], cs...)
	es := strconv.Itoa(mockedExitStatus)
	cmd.Env = []string{"GO_WANT_HELPER_PROCESS=1",
		"STDOUT=" + mockedStdout,
		"EXIT_STATUS=" + es}
	return cmd
}

func TestExecCommandHelper(t *testing.T) {
	if os.Getenv("GO_WANT_HELPER_PROCESS") != "1" {
		return
	}

	fmt.Fprintf(os.Stdout, os.Getenv("STDOUT"))
	i, _ := strconv.Atoi(os.Getenv("EXIT_STATUS"))
	os.Exit(i)
}

func TestDbusLaunchMultilineResponse(t *testing.T) {
	mockedExitStatus = 0
	mockedStdout = `process 7616: D-Bus library appears to be incorrectly set up; failed to read machine uuid: UUID file '/etc/machine-id' should contain a hex string of length 32, not length 0, with no other text
See the manual page for dbus-uuidgen to correct this issue.
DBUS_SESSION_BUS_ADDRESS=unix:abstract=/tmp/dbus-0SO9YZUBGA,guid=ac22f2f3b9d228496b4d4b935cae3417
DBUS_SESSION_BUS_PID=7620
DBUS_SESSION_BUS_WINDOWID=16777217`
	execCommand = fakeExecCommand
	defer func() { execCommand = exec.Command }()
	expOut := ""
	expErr := "dbus: couldn't determine address of session bus"

	out, err := getSessionBusPlatformAddress()
	if out != expOut {
		t.Errorf("Expected %q, got %q", expOut, out)
	}
	if err == nil {
		t.Error("Excepted error, got none")
	} else {
		if err.Error() != expErr {
			t.Errorf("Expected error to be %q, got %q", expErr, err.Error())
		}
	}
}
