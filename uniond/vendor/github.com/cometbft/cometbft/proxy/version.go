package proxy

import (
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/version"
)

// InfoRequest contains all the information for sending
// the abci.InfoRequest message during handshake with the app.
// It contains only compile-time version information.
var InfoRequest = &abci.InfoRequest{
	Version:      version.CMTSemVer,
	BlockVersion: version.BlockProtocol,
	P2PVersion:   version.P2PProtocol,
	AbciVersion:  version.ABCIVersion,
}
