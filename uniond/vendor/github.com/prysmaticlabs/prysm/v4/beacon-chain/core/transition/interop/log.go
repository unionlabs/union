// Package interop contains useful utilities for persisting
// ssz-encoded states and blocks to disk during each state
// transition for development purposes.
package interop

import (
	"github.com/sirupsen/logrus"
)

var log = logrus.WithField("prefix", "interop")
