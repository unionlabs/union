package keeper

import (
	"context"

	"cosmossdk.io/store"
)

func (k Keeper) addDenomFromCreator(ctx context.Context, creator, denom string) {
	store := k.GetCreatorPrefixStore(ctx, creator)
	store.Set([]byte(denom), []byte(denom))
}

func (k Keeper) GetDenomsFromCreator(ctx context.Context, creator string) []string {
	store := k.GetCreatorPrefixStore(ctx, creator)

	iterator := store.Iterator(nil, nil)
	defer iterator.Close()

	denoms := []string{}
	for ; iterator.Valid(); iterator.Next() {
		denoms = append(denoms, string(iterator.Key()))
	}
	return denoms
}

func (k Keeper) GetAllDenomsIterator(ctx context.Context) store.Iterator {
	return k.GetCreatorsPrefixStore(ctx).Iterator(nil, nil)
}

// TODO: Get all denoms a user is the admin of currently
