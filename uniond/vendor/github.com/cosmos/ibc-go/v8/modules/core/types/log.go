package types

import "cosmossdk.io/core/log"

// LogDeferred logs an error in a deferred function call if the returned error is non-nil.
func LogDeferred(logger log.Logger, f func() error) {
	if err := f(); err != nil {
		logger.Error(err.Error())
	}
}
