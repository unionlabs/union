//go:build windows
// +build windows

package winio

import (
	"errors"
	"testing"
)

func TestRunWithUnavailablePrivilege(t *testing.T) {
	err := RunWithPrivilege("SeCreateTokenPrivilege", func() error { return nil })
	var perr *PrivilegeError
	if !errors.As(err, &perr) {
		t.Fatal("expected PrivilegeError")
	}
}

func TestRunWithPrivileges(t *testing.T) {
	err := RunWithPrivilege("SeShutdownPrivilege", func() error { return nil })
	if err != nil {
		t.Fatal(err)
	}
}
