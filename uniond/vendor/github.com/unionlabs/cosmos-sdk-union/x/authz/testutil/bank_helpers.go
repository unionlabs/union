package testutil

import (
	"context"

	bank "cosmossdk.io/x/bank/types"
)

var _ bank.MsgServer = MockBankKeeper{}

func (k MockBankKeeper) Send(ctx context.Context, msg *bank.MsgSend) (*bank.MsgSendResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) Burn(ctx context.Context, msg *bank.MsgBurn) (*bank.MsgBurnResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) MultiSend(ctx context.Context, msg *bank.MsgMultiSend) (*bank.MsgMultiSendResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) UpdateParams(ctx context.Context, req *bank.MsgUpdateParams) (*bank.MsgUpdateParamsResponse, error) {
	return nil, nil
}

func (k MockBankKeeper) SetSendEnabled(ctx context.Context, req *bank.MsgSetSendEnabled) (*bank.MsgSetSendEnabledResponse, error) {
	return nil, nil
}
