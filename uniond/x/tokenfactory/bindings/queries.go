package bindings

import (
	"context"
	"fmt"

	bankkeeper "cosmossdk.io/x/bank/keeper"

	bindingstypes "union/x/tokenfactory/bindings/types"
	tokenfactorykeeper "union/x/tokenfactory/keeper"
)

type QueryPlugin struct {
	bankKeeper         *bankkeeper.BaseKeeper
	tokenFactoryKeeper *tokenfactorykeeper.Keeper
}

// NewQueryPlugin returns a reference to a new QueryPlugin.
func NewQueryPlugin(b *bankkeeper.BaseKeeper, tfk *tokenfactorykeeper.Keeper) *QueryPlugin {
	return &QueryPlugin{
		bankKeeper:         b,
		tokenFactoryKeeper: tfk,
	}
}

// GetDenomAdmin is a query to get denom admin.
func (qp QueryPlugin) GetDenomAdmin(ctx context.Context, denom string) (*bindingstypes.AdminResponse, error) {
	metadata, err := qp.tokenFactoryKeeper.GetAuthorityMetadata(ctx, denom)
	if err != nil {
		return nil, fmt.Errorf("failed to get admin for denom: %s", denom)
	}
	return &bindingstypes.AdminResponse{Admin: metadata.Admin}, nil
}

func (qp QueryPlugin) GetDenomsByCreator(ctx context.Context, creator string) (*bindingstypes.DenomsByCreatorResponse, error) {
	// TODO: validate creator address
	denoms := qp.tokenFactoryKeeper.GetDenomsFromCreator(ctx, creator)
	return &bindingstypes.DenomsByCreatorResponse{Denoms: denoms}, nil
}

func (qp QueryPlugin) GetMetadata(ctx context.Context, denom string) (*bindingstypes.MetadataResponse, error) {
	metadata, found := qp.bankKeeper.GetDenomMetaData(ctx, denom)
	var parsed *bindingstypes.Metadata
	if found {
		parsed = SdkMetadataToWasm(metadata)
	}
	return &bindingstypes.MetadataResponse{Metadata: parsed}, nil
}

func (qp QueryPlugin) GetParams(ctx context.Context) (*bindingstypes.ParamsResponse, error) {
	params := qp.tokenFactoryKeeper.GetParams(ctx)
	return &bindingstypes.ParamsResponse{
		Params: bindingstypes.Params{
			DenomCreationFee: ConvertSdkCoinsToWasmCoins(params.DenomCreationFee),
		},
	}, nil
}
