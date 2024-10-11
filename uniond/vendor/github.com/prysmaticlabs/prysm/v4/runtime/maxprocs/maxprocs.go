// Package maxprocs automatically sets GOMAXPROCS to match the Linux
// container CPU quota, if any. This will not override the environment
// variable of GOMAXPROCS.
package maxprocs

import (
	log "github.com/sirupsen/logrus"
	"go.uber.org/automaxprocs/maxprocs"
)

// Initialize Uber maxprocs.
func init() {
	_, err := maxprocs.Set()
	if err != nil {
		log.WithError(err).Debug("Failed to set maxprocs")
	}
}
