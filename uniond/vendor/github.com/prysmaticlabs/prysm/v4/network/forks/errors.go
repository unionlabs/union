package forks

import "github.com/pkg/errors"

// ErrVersionNotFound indicates the config package couldn't determine the version for an epoch using the fork schedule.
var ErrVersionNotFound = errors.New("could not find an entry in the fork schedule")

// ErrNoPreviousVersion indicates that a version prior to the given version could not be found, because the given version
// is the first one in the list
var ErrNoPreviousVersion = errors.New("no previous version")
