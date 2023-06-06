package wasmtesting

import (
	"errors"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type MockMessageHandler struct {
	DispatchMsgFn func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error)
}

func (m *MockMessageHandler) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
	if m.DispatchMsgFn == nil {
		panic("not expected to be called")
	}
	return m.DispatchMsgFn(ctx, contractAddr, contractIBCPortID, msg)
}

func NewCapturingMessageHandler() (*MockMessageHandler, *[]wasmvmtypes.CosmosMsg) {
	var messages []wasmvmtypes.CosmosMsg
	return &MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
			messages = append(messages, msg)
			// return one data item so that this doesn't cause an error in submessage processing (it takes the first element from data)
			return nil, [][]byte{{1}}, nil
		},
	}, &messages
}

func NewErroringMessageHandler() *MockMessageHandler {
	return &MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
			return nil, nil, errors.New("test, ignore")
		},
	}
}
