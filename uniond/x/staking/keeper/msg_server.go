package keeper

import (
	"context"
	"fmt"

	bn254key "github.com/cosmos/cosmos-sdk/crypto/keys/bn254"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	types "github.com/unionlabs/union/uniond/x/staking/types"
)

var _ types.MsgServer = msgServer{}

type msgServer struct {
	Keeper
}

func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return msgServer{Keeper: keeper}
}

func (k msgServer) CreateUnionValidator(ctx context.Context, req *types.MsgCreateUnionValidator) (*stakingtypes.MsgCreateValidatorResponse, error) {
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
	k.StakingHooks.ProofOfPossessionPassed = true
	defer func() {
		k.StakingHooks.ProofOfPossessionPassed = false
	}()
	return k.stakingMsgServer.CreateValidator(ctx, req.Underlying)
}
