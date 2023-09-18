package simulation

import (
	"math/rand"

	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

// genClasses returns a slice of nft class.
func genClasses(r *rand.Rand, accounts []simtypes.Account) []*nft.Class {
	classes := make([]*nft.Class, len(accounts)-1)
	for i := 0; i < len(accounts)-1; i++ {
		classes[i] = &nft.Class{
			Id:          simtypes.RandStringOfLength(r, 10),
			Name:        simtypes.RandStringOfLength(r, 10),
			Symbol:      simtypes.RandStringOfLength(r, 10),
			Description: simtypes.RandStringOfLength(r, 10),
			Uri:         simtypes.RandStringOfLength(r, 10),
		}
	}
	return classes
}

// genNFT returns a slice of nft.
func genNFT(r *rand.Rand, classID string, accounts []simtypes.Account) []*nft.Entry {
	entries := make([]*nft.Entry, len(accounts)-1)
	for i := 0; i < len(accounts)-1; i++ {
		owner := accounts[i]
		entries[i] = &nft.Entry{
			Owner: owner.Address.String(),
			Nfts: []*nft.NFT{
				{
					ClassId: classID,
					Id:      simtypes.RandStringOfLength(r, 10),
					Uri:     simtypes.RandStringOfLength(r, 10),
				},
			},
		}
	}
	return entries
}

// RandomizedGenState generates a random GenesisState for nft.
func RandomizedGenState(simState *module.SimulationState) {
	var classes []*nft.Class
	simState.AppParams.GetOrGenerate(
		simState.Cdc, "nft", &classes, simState.Rand,
		func(r *rand.Rand) { classes = genClasses(r, simState.Accounts) },
	)

	var entries []*nft.Entry
	simState.AppParams.GetOrGenerate(
		simState.Cdc, "nft", &entries, simState.Rand,
		func(r *rand.Rand) {
			class := classes[r.Int63n(int64(len(classes)))]
			entries = genNFT(r, class.Id, simState.Accounts)
		},
	)

	nftGenesis := &nft.GenesisState{
		Classes: classes,
		Entries: entries,
	}
	simState.GenState[nft.ModuleName] = simState.Cdc.MustMarshalJSON(nftGenesis)
}
