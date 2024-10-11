package accounts

import (
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/transaction"
	banktypes "cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// coinsTransferMsgFunc defines a function that creates a message to send coins from one
// address to the other, and also a message that parses such  response.
// This in most cases will be implemented as a bank.MsgSend creator, but we keep x/accounts independent of bank.
type coinsTransferMsgFunc = func(from, to []byte, coins sdk.Coins) (transaction.Msg, error)

func defaultCoinsTransferMsgFunc(addrCdc address.Codec) coinsTransferMsgFunc {
	return func(from, to []byte, coins sdk.Coins) (transaction.Msg, error) {
		fromAddr, err := addrCdc.BytesToString(from)
		if err != nil {
			return nil, err
		}
		toAddr, err := addrCdc.BytesToString(to)
		if err != nil {
			return nil, err
		}
		return &banktypes.MsgSend{
			FromAddress: fromAddr,
			ToAddress:   toAddr,
			Amount:      coins,
		}, nil
	}
}
