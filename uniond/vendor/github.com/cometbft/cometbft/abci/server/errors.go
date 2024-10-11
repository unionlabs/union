package server

import (
	"fmt"

	"github.com/cometbft/cometbft/abci/types"
)

// ErrUnknownServerType is returned when trying to create a server with invalid transport option.
type ErrUnknownServerType struct {
	ServerType string
}

func (e ErrUnknownServerType) Error() string {
	return "unknown server type " + e.ServerType
}

// ErrConnectionDoesNotExist is returned when trying to access non-existent network connection.
type ErrConnectionDoesNotExist struct {
	ConnID int
}

func (e ErrConnectionDoesNotExist) Error() string {
	return fmt.Sprintf("connection %d does not exist", e.ConnID)
}

type ErrUnknownRequest struct {
	Request types.Request
}

func (e ErrUnknownRequest) Error() string {
	return fmt.Sprintf("unknown request from client: %T", e.Request)
}
