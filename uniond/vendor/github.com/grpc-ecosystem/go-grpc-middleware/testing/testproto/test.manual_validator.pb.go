// Manual code for validation tests.

package mwitkow_testproto

import (
	"errors"
	"math"
)

// Implements the legacy validation interface from protoc-gen-validate.
func (p *PingRequest) Validate() error {
	if p.SleepTimeMs > 10000 {
		return errors.New("cannot sleep for more than 10s")
	}
	return nil
}

// Implements the new validation interface from protoc-gen-validate.
func (p *PingResponse) Validate(bool) error {
	if p.Counter > math.MaxInt16 {
		return errors.New("ping allocation exceeded")
	}
	return nil
}
