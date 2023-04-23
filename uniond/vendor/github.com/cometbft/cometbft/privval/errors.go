package privval

import (
	"errors"
	"fmt"
)

// EndpointTimeoutError occurs when endpoint times out.
type EndpointTimeoutError struct{}

// Implement the net.Error interface.
func (e EndpointTimeoutError) Error() string   { return "endpoint connection timed out" }
func (e EndpointTimeoutError) Timeout() bool   { return true }
func (e EndpointTimeoutError) Temporary() bool { return true }

// Socket errors.
var (
	ErrConnectionTimeout  = EndpointTimeoutError{}
	ErrNoConnection       = errors.New("endpoint is not connected")
	ErrReadTimeout        = errors.New("endpoint read timed out")
	ErrUnexpectedResponse = errors.New("empty response")
	ErrWriteTimeout       = errors.New("endpoint write timed out")
)

// RemoteSignerError allows (remote) validators to include meaningful error
// descriptions in their reply.
type RemoteSignerError struct {
	// TODO(ismail): create an enum of known errors
	Code        int
	Description string
}

func (e *RemoteSignerError) Error() string {
	return fmt.Sprintf("signerEndpoint returned error #%d: %s", e.Code, e.Description)
}
