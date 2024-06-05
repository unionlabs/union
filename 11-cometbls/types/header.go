package types

import (
	"time"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientMessage = (*Header)(nil)

// // ConsensusState returns the updated consensus state associated with the header
// func (h Header) ConsensusState() *ConsensusState {
// 	return &ConsensusState{
// 		Timestamp:          h.GetTime(),
// 		Root:               commitmenttypes.NewMerkleRoot(h.Header.GetAppHash()),
// 		NextValidatorsHash: h.Header.NextValidatorsHash,
// 	}
// }

// ClientType defines that the Header is a Tendermint consensus algorithm
func (Header) ClientType() string {
	// TODO(aeryz): global const
	return "11-cometbls"
}

// GetHeight returns the current height. It returns 0 if the tendermint
// header is nil.
// NOTE: the header.Header is checked to be non nil in ValidateBasic.
func (h Header) GetHeight() exported.Height {
	revision := clienttypes.ParseChainID(h.SignedHeader.Header.ChainID)
	return clienttypes.NewHeight(revision, uint64(h.SignedHeader.Header.Height))
}

// GetTime returns the current block timestamp. It returns a zero time if
// the tendermint header is nil.
// NOTE: the header.Header is checked to be non nil in ValidateBasic.
func (h Header) GetTime() time.Time {
	return h.SignedHeader.Header.Time
}

// ValidateBasic calls the SignedHeader ValidateBasic function and checks
// that validatorsets are not nil.
func (h Header) ValidateBasic() error {
	// TODO(aeryz): implement
	return nil
}
