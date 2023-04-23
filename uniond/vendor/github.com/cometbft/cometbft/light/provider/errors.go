package provider

import (
	"errors"
	"fmt"
)

var (
	// ErrHeightTooHigh is returned when the height is higher than the last
	// block that the provider has. The light client will not remove the provider
	ErrHeightTooHigh = errors.New("height requested is too high")
	// ErrLightBlockNotFound is returned when a provider can't find the
	// requested header (i.e. it has been pruned).
	// The light client will not remove the provider
	ErrLightBlockNotFound = errors.New("light block not found")
	// ErrNoResponse is returned if the provider doesn't respond to the
	// request in a gieven time
	ErrNoResponse = errors.New("client failed to respond")
)

// ErrBadLightBlock is returned when a provider returns an invalid
// light block.
type ErrBadLightBlock struct {
	Reason error
}

func (e ErrBadLightBlock) Error() string {
	return fmt.Sprintf("client provided bad signed header: %s", e.Reason.Error())
}
