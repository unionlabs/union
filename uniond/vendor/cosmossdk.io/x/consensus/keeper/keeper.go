package keeper

import (
	"context"
	"errors"
	"fmt"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttypes "github.com/cometbft/cometbft/types"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	corecontext "cosmossdk.io/core/context"
	"cosmossdk.io/core/event"
	"cosmossdk.io/x/consensus/exported"
	"cosmossdk.io/x/consensus/types"

	"github.com/cosmos/cosmos-sdk/codec"
)

type Keeper struct {
	appmodule.Environment

	authority   string
	ParamsStore collections.Item[cmtproto.ConsensusParams]
}

var _ exported.ConsensusParamSetter = Keeper{}.ParamsStore

// NewKeeper creates a new Keeper instance.
func NewKeeper(cdc codec.BinaryCodec, env appmodule.Environment, authority string) Keeper {
	sb := collections.NewSchemaBuilder(env.KVStoreService)
	return Keeper{
		Environment: env,
		authority:   authority,
		ParamsStore: collections.NewItem(sb, collections.NewPrefix("Consensus"), "params", codec.CollValue[cmtproto.ConsensusParams](cdc)),
	}
}

// GetAuthority returns the authority address for the consensus module.
// This address has the permission to update consensus parameters.
func (k *Keeper) GetAuthority() string {
	return k.authority
}

// InitGenesis initializes the initial state of the module
func (k *Keeper) InitGenesis(ctx context.Context) error {
	value, ok := ctx.Value(corecontext.CometParamsInitInfoKey).(*types.MsgUpdateParams)
	if !ok || value == nil {
		// no error for appv1 and appv2
		return nil
	}

	consensusParams, err := value.ToProtoConsensusParams()
	if err != nil {
		return err
	}

	nextParams, err := k.paramCheck(ctx, consensusParams)
	if err != nil {
		return err
	}

	return k.ParamsStore.Set(ctx, nextParams.ToProto())
}

// Querier

var _ types.QueryServer = Keeper{}

// Params queries params of consensus module
func (k Keeper) Params(ctx context.Context, _ *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	params, err := k.ParamsStore.Get(ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryParamsResponse{Params: &params}, nil
}

// MsgServer

var _ types.MsgServer = Keeper{}

// UpdateParams updates the consensus parameters.
func (k Keeper) UpdateParams(ctx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if k.GetAuthority() != msg.Authority {
		return nil, fmt.Errorf("invalid authority; expected %s, got %s", k.GetAuthority(), msg.Authority)
	}

	consensusParams, err := msg.ToProtoConsensusParams()
	if err != nil {
		return nil, err
	}

	nextParams, err := k.paramCheck(ctx, consensusParams)
	if err != nil {
		return nil, err
	}

	if err := k.ParamsStore.Set(ctx, nextParams.ToProto()); err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		"update_consensus_params",
		event.NewAttribute("authority", msg.Authority),
		event.NewAttribute("parameters", consensusParams.String())); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

// paramCheck validates the consensus params
func (k Keeper) paramCheck(ctx context.Context, consensusParams cmtproto.ConsensusParams) (*cmttypes.ConsensusParams, error) {
	var params cmttypes.ConsensusParams

	paramsProto, err := k.ParamsStore.Get(ctx)
	if err == nil {
		// initialize version params with zero value if not set
		if paramsProto.Version == nil {
			paramsProto.Version = &cmtproto.VersionParams{}
		}
		params = cmttypes.ConsensusParamsFromProto(paramsProto)
	} else if errors.Is(err, collections.ErrNotFound) {
		params = cmttypes.ConsensusParams{}
	} else {
		return nil, err
	}

	nextParams := params.Update(&consensusParams)

	if err := nextParams.ValidateBasic(); err != nil {
		return nil, err
	}

	if err := params.ValidateUpdate(&consensusParams, k.HeaderService.HeaderInfo(ctx).Height); err != nil {
		return nil, err
	}

	return &nextParams, nil
}

// BlockParams returns the maximum gas allowed in a block and the maximum bytes allowed in a block.
func (k Keeper) BlockParams(ctx context.Context) (uint64, uint64, error) {
	params, err := k.ParamsStore.Get(ctx)
	if err != nil {
		return 0, 0, err
	}
	if params.Block == nil {
		return 0, 0, errors.New("block gas is nil")
	}

	return uint64(params.Block.MaxGas), uint64(params.Block.MaxBytes), nil
}

// AppVersion returns the current application version.
func (k Keeper) AppVersion(ctx context.Context) (uint64, error) {
	params, err := k.ParamsStore.Get(ctx)
	if err != nil {
		return 0, err
	}

	if params.Version == nil {
		return 0, errors.New("app version is nil")
	}

	return params.Version.App, nil
}

// ValidatorPubKeyTypes returns the list of public key types that are allowed to be used for validators.
func (k Keeper) ValidatorPubKeyTypes(ctx context.Context) ([]string, error) {
	params, err := k.ParamsStore.Get(ctx)
	if err != nil {
		return nil, err
	}
	if params.Validator == nil {
		return []string{}, errors.New("validator pub key types is nil")
	}

	return params.Validator.PubKeyTypes, nil
}

// EvidenceParams returns the maximum age of evidence, the time duration of the maximum age, and the maximum bytes.
func (k Keeper) EvidenceParams(ctx context.Context) (int64, time.Duration, uint64, error) {
	params, err := k.ParamsStore.Get(ctx)
	if err != nil {
		return 0, 0, 0, err
	}
	if params.Evidence == nil {
		return 0, 0, 0, errors.New("evidence age is nil")
	}

	return params.Evidence.MaxAgeNumBlocks, params.Evidence.MaxAgeDuration, uint64(params.Evidence.MaxBytes), nil
}
