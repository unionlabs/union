package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Verify interface at compile time
var (
	_ sdk.Msg = (*MsgSetWithdrawAddress)(nil)
	_ sdk.Msg = (*MsgWithdrawDelegatorReward)(nil)
	_ sdk.Msg = (*MsgWithdrawValidatorCommission)(nil)
	_ sdk.Msg = (*MsgUpdateParams)(nil)
	_ sdk.Msg = (*MsgCommunityPoolSpend)(nil)
	_ sdk.Msg = (*MsgFundCommunityPool)(nil)
	_ sdk.Msg = (*MsgDepositValidatorRewardsPool)(nil)
)

func NewMsgSetWithdrawAddress(delAddr, withdrawAddr string) *MsgSetWithdrawAddress {
	return &MsgSetWithdrawAddress{
		DelegatorAddress: delAddr,
		WithdrawAddress:  withdrawAddr,
	}
}

func NewMsgWithdrawDelegatorReward(delAddr, valAddr string) *MsgWithdrawDelegatorReward {
	return &MsgWithdrawDelegatorReward{
		DelegatorAddress: delAddr,
		ValidatorAddress: valAddr,
	}
}

func NewMsgWithdrawValidatorCommission(valAddr string) *MsgWithdrawValidatorCommission {
	return &MsgWithdrawValidatorCommission{
		ValidatorAddress: valAddr,
	}
}

// NewMsgFundCommunityPool returns a new MsgFundCommunityPool with a sender and
// a funding amount.
func NewMsgFundCommunityPool(amount sdk.Coins, depositor string) *MsgFundCommunityPool {
	return &MsgFundCommunityPool{
		Amount:    amount,
		Depositor: depositor,
	}
}

// NewMsgDepositValidatorRewardsPool returns a new MsgDepositValidatorRewardsPool
// with a depositor and a funding amount.
func NewMsgDepositValidatorRewardsPool(depositor, valAddr string, amount sdk.Coins) *MsgDepositValidatorRewardsPool {
	return &MsgDepositValidatorRewardsPool{
		Amount:           amount,
		Depositor:        depositor,
		ValidatorAddress: valAddr,
	}
}
