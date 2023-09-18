package types

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestValidateGenesis(t *testing.T) {
	testCases := []struct {
		name     string
		malleate func(*GenesisState)
		expPass  bool
	}{
		{
			name:     "default",
			malleate: func(_ *GenesisState) {},
			expPass:  true,
		},
		{
			name: "valid genesis state",
			malleate: func(genState *GenesisState) {
				genState.Index = 10
				genOwner := GenesisOwners{
					Index:       1,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: true,
		},
		{
			name: "initial index is 0",
			malleate: func(genState *GenesisState) {
				genState.Index = 0
				genOwner := GenesisOwners{
					Index:       0,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},

		{
			name: "blank owner module",
			malleate: func(genState *GenesisState) {
				genState.Index = 1
				genOwner := GenesisOwners{
					Index:       1,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "blank owner name",
			malleate: func(genState *GenesisState) {
				genState.Index = 1
				genOwner := GenesisOwners{
					Index:       1,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "ibc", Name: ""}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "index above range",
			malleate: func(genState *GenesisState) {
				genState.Index = 10
				genOwner := GenesisOwners{
					Index:       12,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "index below range",
			malleate: func(genState *GenesisState) {
				genState.Index = 10
				genOwner := GenesisOwners{
					Index:       0,
					IndexOwners: CapabilityOwners{[]Owner{{Module: "ibc", Name: "port/transfer"}}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
		{
			name: "owners are empty",
			malleate: func(genState *GenesisState) {
				genState.Index = 10
				genOwner := GenesisOwners{
					Index:       0,
					IndexOwners: CapabilityOwners{[]Owner{}},
				}

				genState.Owners = append(genState.Owners, genOwner)
			},
			expPass: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		genState := DefaultGenesis()
		tc.malleate(genState)
		err := genState.Validate()
		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}
