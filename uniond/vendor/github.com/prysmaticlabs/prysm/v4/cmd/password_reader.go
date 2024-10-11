package cmd

import (
	"os"

	"golang.org/x/crypto/ssh/terminal"
)

// PasswordReader reads a password from a mock or stdin.
type PasswordReader interface {
	ReadPassword() (string, error)
}

// StdInPasswordReader reads a password from stdin.
type StdInPasswordReader struct {
}

// ReadPassword reads a password from stdin.
func (StdInPasswordReader) ReadPassword() (string, error) {
	pwd, err := terminal.ReadPassword(int(os.Stdin.Fd()))
	return string(pwd), err
}
