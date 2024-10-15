package keeper

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"strings"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ types.MsgServer = msgServer{}

type msgServer struct {
	ak AccountKeeper
}

// NewMsgServerImpl returns an implementation of the x/auth MsgServer interface.
func NewMsgServerImpl(ak AccountKeeper) types.MsgServer {
	return &msgServer{
		ak: ak,
	}
}

func (ms msgServer) NonAtomicExec(ctx context.Context, msg *types.MsgNonAtomicExec) (*types.MsgNonAtomicExecResponse, error) {
	if msg.Signer == "" {
		return nil, errors.New("empty signer address string is not allowed")
	}

	signer, err := ms.ak.AddressCodec().StringToBytes(msg.Signer)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid signer address: %s", err)
	}

	if len(msg.Msgs) == 0 {
		return nil, sdkerrors.ErrInvalidRequest.Wrapf("messages cannot be empty")
	}

	msgs, err := msg.GetMessages()
	if err != nil {
		return nil, err
	}

	results, err := ms.ak.NonAtomicMsgsExec(ctx, signer, msgs)
	if err != nil {
		return nil, err
	}

	return &types.MsgNonAtomicExecResponse{
		Results: results,
	}, nil
}

func (ms msgServer) UpdateParams(ctx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if ms.ak.authority != msg.Authority {
		return nil, fmt.Errorf(
			"expected authority account as only signer for proposal message; invalid authority; expected %s, got %s",
			ms.ak.authority, msg.Authority)
	}

	if err := msg.Params.Validate(); err != nil {
		return nil, err
	}

	if err := ms.ak.Params.Set(ctx, msg.Params); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

func (ms msgServer) MigrateAccount(ctx context.Context, msg *types.MsgMigrateAccount) (*types.MsgMigrateAccountResponse, error) {
	signer, err := ms.ak.AddressCodec().StringToBytes(msg.Signer)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid signer address: %s", err)
	}

	acc := ms.ak.GetAccount(ctx, signer)
	if acc == nil {
		return nil, sdkerrors.ErrUnknownAddress.Wrapf("account %s does not exist", signer)
	}

	// check if account type is valid or not
	_, isBaseAccount := (acc).(*types.BaseAccount)
	if !isBaseAccount {
		return nil, status.Error(codes.InvalidArgument, "only BaseAccount can be migrated")
	}

	// unwrap any msg
	initMsg, err := unpackAnyRaw(msg.AccountInitMsg)
	if err != nil {
		return nil, err
	}

	initResp, err := ms.ak.AccountsModKeeper.MigrateLegacyAccount(ctx, signer, acc.GetAccountNumber(), msg.AccountType, initMsg)
	if err != nil {
		return nil, err
	}

	// account is then removed from state
	ms.ak.RemoveAccount(ctx, acc)

	initRespAny, err := codectypes.NewAnyWithValue(initResp)
	if err != nil {
		return nil, err
	}

	return &types.MsgMigrateAccountResponse{InitResponse: initRespAny}, nil
}

func unpackAnyRaw(m *codectypes.Any) (gogoproto.Message, error) {
	if m == nil {
		return nil, fmt.Errorf("cannot unpack nil any")
	}
	split := strings.Split(m.TypeUrl, "/")
	name := split[len(split)-1]
	typ := gogoproto.MessageType(name)
	if typ == nil {
		return nil, fmt.Errorf("no message type found for %s", name)
	}
	concreteMsg := reflect.New(typ.Elem()).Interface().(gogoproto.Message)
	err := gogoproto.Unmarshal(m.Value, concreteMsg)
	if err != nil {
		return nil, err
	}

	return concreteMsg, nil
}
