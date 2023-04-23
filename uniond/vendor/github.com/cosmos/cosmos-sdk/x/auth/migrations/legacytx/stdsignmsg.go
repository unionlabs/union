package legacytx

import (
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ types.UnpackInterfacesMessage = StdSignMsg{}

// StdSignMsg is a convenience structure for passing along a Msg with the other
// requirements for a StdSignDoc before it is signed. For use in the CLI.
type StdSignMsg struct {
	ChainID       string    `json:"chain_id" yaml:"chain_id"`
	AccountNumber uint64    `json:"account_number" yaml:"account_number"`
	Sequence      uint64    `json:"sequence" yaml:"sequence"`
	TimeoutHeight uint64    `json:"timeout_height" yaml:"timeout_height"`
	Fee           StdFee    `json:"fee" yaml:"fee"`
	Msgs          []sdk.Msg `json:"msgs" yaml:"msgs"`
	Memo          string    `json:"memo" yaml:"memo"`
}

// get message bytes
func (msg StdSignMsg) Bytes() []byte {
	return StdSignBytes(msg.ChainID, msg.AccountNumber, msg.Sequence, msg.TimeoutHeight, msg.Fee, msg.Msgs, msg.Memo, nil)
}

func (msg StdSignMsg) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	for _, m := range msg.Msgs {
		err := types.UnpackInterfaces(m, unpacker)
		if err != nil {
			return err
		}
	}

	return nil
}
