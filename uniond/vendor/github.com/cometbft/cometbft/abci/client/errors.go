package abcicli

import (
	"fmt"

	"github.com/cometbft/cometbft/abci/types"
)

// ErrUnknownAbciTransport is returned when trying to create a client with an invalid transport option.
type ErrUnknownAbciTransport struct {
	Transport string
}

func (e ErrUnknownAbciTransport) Error() string {
	return "unknown abci transport: " + e.Transport
}

type ErrUnexpectedResponse struct {
	Response types.Response
	Reason   string
}

func (e ErrUnexpectedResponse) Error() string {
	return fmt.Sprintf("unexpected response %T: %s", e.Response.Value, e.Reason)
}
