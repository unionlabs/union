package testutil

import (
	"context"

	bank "github.com/cosmos/cosmos-sdk/x/bank/types"
)

var _ bank.MsgServer = MockBankKeeper{}

func (k MockBankKeeper) Send(goCtx context.Context, msg *bank.MsgSend) (*bank.MsgSendResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) MultiSend(goCtx context.Context, msg *bank.MsgMultiSend) (*bank.MsgMultiSendResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) UpdateParams(goCtx context.Context, req *bank.MsgUpdateParams) (*bank.MsgUpdateParamsResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) SetSendEnabled(goCtx context.Context, req *bank.MsgSetSendEnabled) (*bank.MsgSetSendEnabledResponse, error) {
	return nil, nil
}
