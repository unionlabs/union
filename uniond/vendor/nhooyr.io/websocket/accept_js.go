package websocket

import (
	"errors"
	"net/http"
)

// AcceptOptions represents Accept's options.
type AcceptOptions struct {
	Subprotocols         []string
	InsecureSkipVerify   bool
	OriginPatterns       []string
	CompressionMode      CompressionMode
	CompressionThreshold int
}

// Accept is stubbed out for Wasm.
func Accept(w http.ResponseWriter, r *http.Request, opts *AcceptOptions) (*Conn, error) {
	return nil, errors.New("unimplemented")
}
