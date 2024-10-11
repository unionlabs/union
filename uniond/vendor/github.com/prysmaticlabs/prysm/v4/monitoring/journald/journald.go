//go:build !linux

package journald

import (
	"fmt"
)

// Enable returns an error on non-Linux systems
func Enable() error {
	return fmt.Errorf("journald is not supported in this platform")
}
