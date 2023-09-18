package keeper

import (
	"sort"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

// InitGenesis initializes the nft module's genesis state from a given
// genesis state.
func (k Keeper) InitGenesis(ctx sdk.Context, data *nft.GenesisState) {
	for _, class := range data.Classes {
		if err := k.SaveClass(ctx, *class); err != nil {
			panic(err)
		}
	}
	for _, entry := range data.Entries {
		for _, nft := range entry.Nfts {
			owner := sdk.MustAccAddressFromBech32(entry.Owner)

			if err := k.Mint(ctx, *nft, owner); err != nil {
				panic(err)
			}
		}
	}
}

// ExportGenesis returns a GenesisState for a given context.
func (k Keeper) ExportGenesis(ctx sdk.Context) *nft.GenesisState {
	classes := k.GetClasses(ctx)
	nftMap := make(map[string][]*nft.NFT)
	for _, class := range classes {
		nfts := k.GetNFTsOfClass(ctx, class.Id)
		for i, n := range nfts {
			owner := k.GetOwner(ctx, n.ClassId, n.Id)
			nftArr, ok := nftMap[owner.String()]
			if !ok {
				nftArr = make([]*nft.NFT, 0)
			}
			nftMap[owner.String()] = append(nftArr, &nfts[i])
		}
	}

	owners := make([]string, 0, len(nftMap))
	for owner := range nftMap {
		owners = append(owners, owner)
	}
	sort.Strings(owners)

	entries := make([]*nft.Entry, 0, len(nftMap))
	for _, owner := range owners {
		entries = append(entries, &nft.Entry{
			Owner: owner,
			Nfts:  nftMap[owner],
		})
	}
	return &nft.GenesisState{
		Classes: classes,
		Entries: entries,
	}
}
