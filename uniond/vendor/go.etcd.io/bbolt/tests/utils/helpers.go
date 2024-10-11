package utils

import (
	"flag"
	"fmt"
	"os"
)

var enableRoot bool

func init() {
	flag.BoolVar(&enableRoot, "test.root", false, "enable tests that require root")
}

// RequiresRoot requires root and the test.root flag has been set.
func RequiresRoot() {
	if !enableRoot {
		fmt.Fprintln(os.Stderr, "Skip tests that require root")
		os.Exit(0)
	}

	if os.Getuid() != 0 {
		fmt.Fprintln(os.Stderr, "This test must be run as root.")
		os.Exit(1)
	}
}
