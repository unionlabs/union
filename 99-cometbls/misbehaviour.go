package cometbls

import (
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientMessage = (*Misbehaviour)(nil)

// ClientType is Tendermint light client
func (Misbehaviour) ClientType() string {
	return "99-cometbls"
}

// ValidateBasic implements Misbehaviour interface
func (misbehaviour Misbehaviour) ValidateBasic() error {
	return nil
}
