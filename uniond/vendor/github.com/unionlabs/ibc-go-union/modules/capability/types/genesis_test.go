package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/modules/capability/types"
)

func TestValidateGenesis(t *testing.T) {
	testCases := []struct {
		name     string
		malleate func(*types.GenesisState)
		expPass  bool
	}{
		{
			name:     "default",
			malleate: func(_ *types.GenesisState) {},
			expPass:  true,
		},
		{
			name: "valid genesis state",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 10
				genOwner := types.GenesisOwners{
					Index:       1,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: true,
		},
		{
			name: "initial index is 0",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 0
				genOwner := types.GenesisOwners{
					Index:       0,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},

		{
			name: "blank owner module",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 1
				genOwner := types.GenesisOwners{
					Index:       1,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "blank owner name",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 1
				genOwner := types.GenesisOwners{
					Index:       1,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "ibc", Name: ""}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "index above range",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 10
				genOwner := types.GenesisOwners{
					Index:       12,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "index below range",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 10
				genOwner := types.GenesisOwners{
					Index:       0,
					IndexOwners: types.CapabilityOwners{[]types.Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "owners are empty",
			malleate: func(genState *types.GenesisState) {
				genState.Index = 10
				genOwner := types.GenesisOwners{
					Index:       0,
					IndexOwners: types.CapabilityOwners{[]types.Owner{}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		genState := types.DefaultGenesis()
		tc.malleate(genState)
		err := genState.Validate()
		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}
