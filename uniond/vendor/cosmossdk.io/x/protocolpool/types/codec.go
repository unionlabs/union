package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations(
		(*coretransaction.Msg)(nil),
		&MsgFundCommunityPool{},
		&MsgCommunityPoolSpend{},
		&MsgSubmitBudgetProposal{},
		&MsgClaimBudget{},
		&MsgCreateContinuousFund{},
		&MsgCancelContinuousFund{},
		&MsgWithdrawContinuousFund{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
