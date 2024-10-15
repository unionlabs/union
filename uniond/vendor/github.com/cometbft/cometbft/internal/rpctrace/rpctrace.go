package rpctrace

import "github.com/google/uuid"

// New returns a randomly generated string which can be used to assist in
// tracing RPC errors.
func New() (string, error) {
	id := uuid.New()
	return id.String(), nil
}
