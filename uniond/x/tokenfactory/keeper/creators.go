package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (k Keeper) addDenomFromCreator(ctx sdk.Context, creator, denom string) {
	store := k.GetCreatorPrefixStore(ctx, creator)
	store.Set([]byte(denom), []byte(denom))
}

func (k Keeper) GetDenomsFromCreator(ctx sdk.Context, creator string) []string {
	store := k.GetCreatorPrefixStore(ctx, creator)

	iterator := store.Iterator(nil, nil)
	defer iterator.Close()

	denoms := []string{}
	for ; iterator.Valid(); iterator.Next() {
		denoms = append(denoms, string(iterator.Key()))
	}
	return denoms
}

func (k Keeper) GetAllDenomsIterator(ctx sdk.Context) sdk.Iterator {
	return k.GetCreatorsPrefixStore(ctx).Iterator(nil, nil)
}

// TODO: Get all denoms a user is the admin of currently
