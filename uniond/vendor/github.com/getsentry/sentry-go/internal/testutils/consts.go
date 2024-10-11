package testutils

import (
	"os"
	"time"
)

func IsCI() bool {
	return os.Getenv("CI") != ""
}

func FlushTimeout() time.Duration {
	if IsCI() {
		// CI is very overloaded so we need to allow for a long wait time.
		return 5 * time.Second
	}

	return time.Second
}
