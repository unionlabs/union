//go:build linux

package journald

import (
	"io"

	"github.com/coreos/go-systemd/journal"
	"github.com/sirupsen/logrus"
)

// Enable adds the Journal hook if journal is enabled
// Sets log output to ioutil.Discard so stdout isn't captured.
func Enable() error {
	if !journal.Enabled() {
		logrus.Warning("Journal not available but user requests we log to it. Ignoring")
	} else {
		logrus.AddHook(&JournalHook{})
		logrus.SetOutput(io.Discard)
	}
	return nil
}
