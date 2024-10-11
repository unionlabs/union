package store

import "errors"

// ErrLightBlockNotFound is returned when a store does not have the
// requested header.
var ErrLightBlockNotFound = errors.New("light block not found")
