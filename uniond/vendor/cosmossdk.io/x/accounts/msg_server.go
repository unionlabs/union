package accounts

import (
	"context"
	"fmt"

	"cosmossdk.io/core/event"
	"cosmossdk.io/x/accounts/internal/implementation"
	v1 "cosmossdk.io/x/accounts/v1"
)

var _ v1.MsgServer = msgServer{}

func NewMsgServer(k Keeper) v1.MsgServer {
	return &msgServer{k}
}

type msgServer struct {
	k Keeper
}

func (m msgServer) Init(ctx context.Context, request *v1.MsgInit) (*v1.MsgInitResponse, error) {
	resp, accAddr, err := m.k.initFromMsg(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("unable to initialize account: %w", err)
	}

	// encode the address
	accAddrString, err := m.k.addressCodec.BytesToString(accAddr)
	if err != nil {
		return nil, err
	}

	eventManager := m.k.EventService.EventManager(ctx)
	err = eventManager.EmitKV(
		"account_creation",
		event.NewAttribute("address", accAddrString),
	)
	if err != nil {
		return nil, err
	}

	anyResp, err := implementation.PackAny(resp)
	if err != nil {
		return nil, err
	}
	return &v1.MsgInitResponse{
		AccountAddress: accAddrString,
		Response:       anyResp,
	}, nil
}

func (m msgServer) Execute(ctx context.Context, execute *v1.MsgExecute) (*v1.MsgExecuteResponse, error) {
	// decode sender address
	senderAddr, err := m.k.addressCodec.StringToBytes(execute.Sender)
	if err != nil {
		return nil, err
	}
	// decode the target address
	targetAddr, err := m.k.addressCodec.StringToBytes(execute.Target)
	if err != nil {
		return nil, err
	}

	// decode message bytes into the concrete boxed message type
	req, err := implementation.UnpackAnyRaw(execute.Message)
	if err != nil {
		return nil, err
	}

	// run account execution logic
	resp, err := m.k.Execute(ctx, targetAddr, senderAddr, req, execute.Funds)
	if err != nil {
		return nil, err
	}

	// encode the response
	respAny, err := implementation.PackAny(resp)
	if err != nil {
		return nil, err
	}
	return &v1.MsgExecuteResponse{
		Response: respAny,
	}, nil
}

func (m msgServer) ExecuteBundle(ctx context.Context, req *v1.MsgExecuteBundle) (*v1.MsgExecuteBundleResponse, error) {
	panic("impl")
}
