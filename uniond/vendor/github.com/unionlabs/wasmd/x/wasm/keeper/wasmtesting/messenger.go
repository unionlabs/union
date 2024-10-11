package wasmtesting

import (
	"errors"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type MockMessageHandler struct {
	DispatchMsgFn func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error)
}

func (m *MockMessageHandler) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
	if m.DispatchMsgFn == nil {
		panic("not expected to be called")
	}
	return m.DispatchMsgFn(ctx, contractAddr, contractIBCPortID, msg)
}

func NewCapturingMessageHandler() (*MockMessageHandler, *[]wasmvmtypes.CosmosMsg) {
	var messages []wasmvmtypes.CosmosMsg
	return &MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
			messages = append(messages, msg)
			// return one data item so that this doesn't cause an error in submessage processing (it takes the first element from data)
			return nil, [][]byte{{1}}, [][]*codectypes.Any{}, nil
		},
	}, &messages
}

func NewErroringMessageHandler() *MockMessageHandler {
	return &MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
			return nil, nil, [][]*codectypes.Any{}, errors.New("test, ignore")
		},
	}
}
