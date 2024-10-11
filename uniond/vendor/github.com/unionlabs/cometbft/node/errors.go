package node

import (
	"fmt"
)

// ErrorLoadOrGenFilePV is returned when the node fails to load or generate priv validator file.
type ErrorLoadOrGenFilePV struct {
	Err       error
	KeyFile   string
	StateFile string
}

func (e ErrorLoadOrGenFilePV) Error() string {
	return fmt.Sprintf("failed to load or generate privval file; "+
		"key file %s, state file %s: %v", e.KeyFile, e.StateFile, e.Err)
}

func (e ErrorLoadOrGenFilePV) Unwrap() error {
	return e.Err
}
