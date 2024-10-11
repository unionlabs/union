package fdlimits

import (
	"github.com/ethereum/go-ethereum/common/fdlimit"
	"github.com/sirupsen/logrus"
)

var log = logrus.WithField("prefix", "fdlimits")

// SetMaxFdLimits is a wrapper around a few go-ethereum methods to allow prysm to
// set its file descriptor limits at the maximum possible value.
func SetMaxFdLimits() error {
	curr, err := fdlimit.Current()
	if err != nil {
		return err
	}
	max, err := fdlimit.Maximum()
	if err != nil {
		return err
	}
	raisedVal, err := fdlimit.Raise(uint64(max))
	if err != nil {
		return err
	}
	log.Debugf("Updated file descriptor limit to %d from %d", raisedVal, curr)
	return nil
}
