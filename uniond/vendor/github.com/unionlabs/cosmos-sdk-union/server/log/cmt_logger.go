package server

import (
	cmtlog "github.com/cometbft/cometbft/libs/log"

	"cosmossdk.io/log"
)

var _ cmtlog.Logger = (*CometLoggerWrapper)(nil)

// CometLoggerWrapper provides a wrapper around a cosmossdk.io/log instance.
// It implements CometBFT's Logger interface.
type CometLoggerWrapper struct {
	log.Logger
}

// With returns a new wrapped logger with additional context provided by a set
// of key/value tuples. The number of tuples must be even and the key of the
// tuple must be a string.
func (cmt CometLoggerWrapper) With(keyVals ...interface{}) cmtlog.Logger {
	logger := cmt.Logger.With(keyVals...)
	return CometLoggerWrapper{logger}
}
