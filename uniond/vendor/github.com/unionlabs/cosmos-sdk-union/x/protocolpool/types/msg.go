package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ sdk.Msg = (*MsgFundCommunityPool)(nil)
	_ sdk.Msg = (*MsgCommunityPoolSpend)(nil)
)

// NewMsgFundCommunityPool returns a new MsgFundCommunityPool with a sender and
// a funding amount.
func NewMsgFundCommunityPool(amount sdk.Coins, depositor string) *MsgFundCommunityPool {
	return &MsgFundCommunityPool{
		Amount:    amount,
		Depositor: depositor,
	}
}

// NewMsgCommunityPoolSpend returns a new CommunityPoolSpend with authority, recipient and
// a spending amount.
func NewMsgCommunityPoolSpend(amount sdk.Coins, authority, recipient string) *MsgCommunityPoolSpend {
	return &MsgCommunityPoolSpend{
		Authority: authority,
		Recipient: recipient,
		Amount:    amount,
	}
}
