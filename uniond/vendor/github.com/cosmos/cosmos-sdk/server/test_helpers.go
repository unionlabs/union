package server

import (
	"fmt"
	"net"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Get a free address for a test tendermint server
// protocol is either tcp, http, etc
func FreeTCPAddr() (addr, port string, err error) {
	l, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		return "", "", err
	}

	if err := l.Close(); err != nil {
		return "", "", sdkerrors.Wrap(err, "couldn't close the listener")
	}

	portI := l.Addr().(*net.TCPAddr).Port
	port = fmt.Sprintf("%d", portI)
	addr = fmt.Sprintf("tcp://0.0.0.0:%s", port)
	return
}
