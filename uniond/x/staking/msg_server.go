package staking

import (
	"context"
	"fmt"

	anytypes "github.com/cosmos/gogoproto/types/any"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	bn254key "github.com/cosmos/cosmos-sdk/crypto/keys/bn254"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"

	types "cosmossdk.io/x/staking/types"
)

type msgServer struct {
	stakingMsgServer types.MsgServer
	StakingHooks     *Hooks
}

var (
	_ MsgServer = &msgServer{}

	_ sdk.Msg                          = &MsgCreateUnionValidator{}
	_ anytypes.UnpackInterfacesMessage = (*MsgCreateUnionValidator)(nil)
)

func (msg MsgCreateUnionValidator) UnpackInterfaces(unpacker anytypes.AnyUnpacker) error {
	return msg.Underlying.UnpackInterfaces(unpacker)
}

func NewMsgServerImpl(baseApp *baseapp.BaseApp, stakingMsgServer types.MsgServer) *msgServer {
	return &msgServer{
		stakingMsgServer: stakingMsgServer,
		StakingHooks: &Hooks{
			ProofOfPossessionPassed: false,
			baseApp:                 baseApp,
		},
	}
}

func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
		&MsgCreateUnionValidator{},
	)
	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}

func (m *msgServer) CreateUnionValidator(ctx context.Context, req *MsgCreateUnionValidator) (*types.MsgCreateValidatorResponse, error) {
	if req.ValidatorAddress != req.Underlying.ValidatorAddress {
		return nil, fmt.Errorf("validator_address != underlying.validator_address")
	}
	if req.Underlying.Pubkey.TypeUrl != "/cosmos.crypto.bn254.PubKey" {
		return nil, fmt.Errorf("expected /cosmos.crypto.bn254.PubKey: got %s", req.Underlying.Pubkey.TypeUrl)
	}
	var pk bn254key.PubKey
	err := pk.Unmarshal(req.Underlying.Pubkey.Value)
	if err != nil {
		return nil, fmt.Errorf("invalid bn254 key: %w", err)
	}
	if !pk.VerifySignature(pk.Bytes(), req.ProofOfPossession) {
		return nil, fmt.Errorf("invalid proof of possession")
	}
	m.StakingHooks.ProofOfPossessionPassed = true
	defer func() {
		m.StakingHooks.ProofOfPossessionPassed = false
	}()
	return m.stakingMsgServer.CreateValidator(ctx, req.Underlying)
}
