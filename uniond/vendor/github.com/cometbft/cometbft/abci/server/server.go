/*
Package server is used to start a new ABCI server.

It contains two server implementation:
  - gRPC server
  - socket server
*/
package server

import (
	"fmt"

	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/service"
)

// NewServer is a utility function for out of process applications to set up either a socket or
// grpc server that can listen to requests from the equivalent Tendermint client
func NewServer(protoAddr, transport string, app types.Application) (service.Service, error) {
	var s service.Service
	var err error
	switch transport {
	case "socket":
		s = NewSocketServer(protoAddr, app)
	case "grpc":
		s = NewGRPCServer(protoAddr, app)
	default:
		err = fmt.Errorf("unknown server type %s", transport)
	}
	return s, err
}
