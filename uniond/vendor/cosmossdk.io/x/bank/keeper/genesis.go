package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

// InitGenesis initializes the bank module's state from a given genesis state.
func (k BaseKeeper) InitGenesis(ctx context.Context, genState *types.GenesisState) error {
	var err error
	if err = k.SetParams(ctx, genState.Params); err != nil {
		return err
	}

	for _, se := range genState.GetAllSendEnabled() {
		k.SetSendEnabled(ctx, se.Denom, se.Enabled)
	}
	totalSupplyMap := sdk.NewMapCoins(sdk.Coins{})

	genState.Balances, err = types.SanitizeGenesisBalances(genState.Balances, k.ak.AddressCodec())
	if err != nil {
		return err
	}

	for _, balance := range genState.Balances {
		addr := balance.GetAddress()
		bz, err := k.ak.AddressCodec().StringToBytes(addr)
		if err != nil {
			return err
		}

		for _, coin := range balance.Coins {
			err := k.Balances.Set(ctx, collections.Join(sdk.AccAddress(bz), coin.Denom), coin.Amount)
			if err != nil {
				return err
			}
		}

		totalSupplyMap.Add(balance.Coins...)
	}
	totalSupply := totalSupplyMap.ToCoins()

	if !genState.Supply.Empty() && !genState.Supply.Equal(totalSupply) {
		return fmt.Errorf("genesis supply is incorrect, expected %v, got %v", genState.Supply, totalSupply)
	}

	for _, supply := range totalSupply {
		k.setSupply(ctx, supply)
	}

	for _, meta := range genState.DenomMetadata {
		k.SetDenomMetaData(ctx, meta)
	}
	return nil
}

// ExportGenesis returns the bank module's genesis state.
func (k BaseKeeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	totalSupply, _, err := k.GetPaginatedTotalSupply(ctx, &query.PageRequest{Limit: query.PaginationMaxLimit})
	if err != nil {
		return nil, fmt.Errorf("unable to fetch total supply %w", err)
	}

	rv := types.NewGenesisState(
		k.GetParams(ctx),
		k.GetAccountsBalances(ctx),
		totalSupply,
		k.GetAllDenomMetaData(ctx),
		k.GetAllSendEnabledEntries(ctx),
	)
	return rv, nil
}
