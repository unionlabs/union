package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestGenesisStateValidate(t *testing.T) {
	testCases := []struct {
		name         string
		genesisState GenesisState
		expErr       bool
	}{
		{
			"valid genesisState",
			GenesisState{
				Params: DefaultParams(),
				Balances: []Balance{
					{
						Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
						Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
					},
				},
				Supply: sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
				DenomMetadata: []Metadata{
					{
						Name:        "Cosmos Hub Atom",
						Symbol:      "ATOM",
						Description: "The native staking token of the Cosmos Hub.",
						DenomUnits: []*DenomUnit{
							{"uatom", uint32(0), []string{"microatom"}},
							{"matom", uint32(3), []string{"milliatom"}},
							{"atom", uint32(6), nil},
						},
						Base:    "uatom",
						Display: "atom",
					},
				},
			},
			false,
		},
		{"empty genesisState", GenesisState{}, false},
		{
			"invalid params ",
			GenesisState{
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"", true},
					},
				},
			},
			true,
		},
		{
			"dup balances",
			GenesisState{
				Balances: []Balance{
					{
						Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
						Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
					},
					{
						Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
						Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
					},
				},
			},
			true,
		},
		{
			"0  balance",
			GenesisState{
				Balances: []Balance{
					{
						Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
					},
				},
			},
			false,
		},
		{
			"dup Metadata",
			GenesisState{
				DenomMetadata: []Metadata{
					{
						Name:        "Cosmos Hub Atom",
						Symbol:      "ATOM",
						Description: "The native staking token of the Cosmos Hub.",
						DenomUnits: []*DenomUnit{
							{"uatom", uint32(0), []string{"microatom"}},
							{"matom", uint32(3), []string{"milliatom"}},
							{"atom", uint32(6), nil},
						},
						Base:    "uatom",
						Display: "atom",
					},
					{
						Name:        "Cosmos Hub Atom",
						Symbol:      "ATOM",
						Description: "The native staking token of the Cosmos Hub.",
						DenomUnits: []*DenomUnit{
							{"uatom", uint32(0), []string{"microatom"}},
							{"matom", uint32(3), []string{"milliatom"}},
							{"atom", uint32(6), nil},
						},
						Base:    "uatom",
						Display: "atom",
					},
				},
			},
			true,
		},
		{
			"invalid Metadata",
			GenesisState{
				DenomMetadata: []Metadata{
					{
						Name:        "Cosmos Hub Atom",
						Symbol:      "ATOM",
						Description: "The native staking token of the Cosmos Hub.",
						DenomUnits: []*DenomUnit{
							{"uatom", uint32(0), []string{"microatom"}},
							{"matom", uint32(3), []string{"milliatom"}},
							{"atom", uint32(6), nil},
						},
						Base:    "",
						Display: "",
					},
				},
			},
			true,
		},
		{
			"invalid supply",
			GenesisState{
				Supply: sdk.Coins{sdk.Coin{Denom: "", Amount: math.OneInt()}},
			},
			true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(tt *testing.T) {
			err := tc.genesisState.Validate()

			if tc.expErr {
				require.Error(tt, err)
			} else {
				require.NoError(tt, err)
			}
		})
	}
}

func TestMigrateSendEnabled(t *testing.T) {
	tests := []struct {
		name     string
		oldState *GenesisState
		newState *GenesisState
	}{
		{
			name: "Balances supply metadata all unchanged",
			oldState: &GenesisState{
				Params: Params{},
				Balances: []Balance{{
					Address: "balance1",
					Coins:   sdk.Coins{sdk.NewCoin("balance1coin", math.NewInt(8))},
				}},
				Supply: sdk.Coins{sdk.NewCoin("supplycoin", math.NewInt(800))},
				DenomMetadata: []Metadata{{
					Description: "metadesk",
					DenomUnits:  nil,
					Base:        "meta",
					Display:     "meta",
					Name:        "foo",
					Symbol:      "META",
					URI:         "",
					URIHash:     "",
				}},
			},
			newState: &GenesisState{
				Params: Params{},
				Balances: []Balance{{
					Address: "balance1",
					Coins:   sdk.Coins{sdk.NewCoin("balance1coin", math.NewInt(8))},
				}},
				Supply: sdk.Coins{sdk.NewCoin("supplycoin", math.NewInt(800))},
				DenomMetadata: []Metadata{{
					Description: "metadesk",
					DenomUnits:  nil,
					Base:        "meta",
					Display:     "meta",
					Name:        "foo",
					Symbol:      "META",
					URI:         "",
					URIHash:     "",
				}},
			},
		},

		{
			name: "default send enabled true not changed",
			oldState: &GenesisState{
				Params: Params{DefaultSendEnabled: true},
			},
			newState: &GenesisState{
				Params: Params{DefaultSendEnabled: true},
			},
		},
		{
			name: "default send enabled false not changed",
			oldState: &GenesisState{
				Params: Params{DefaultSendEnabled: false, SendEnabled: []*SendEnabled{}},
			},
			newState: &GenesisState{
				Params: Params{DefaultSendEnabled: false},
			},
		},
		{
			name: "send enabled entries moved",
			oldState: &GenesisState{
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"movecointrue", true},
						{"movecoinfalse", false},
					},
				},
			},
			newState: &GenesisState{
				Params: Params{},
				SendEnabled: []SendEnabled{
					{"movecointrue", true},
					{"movecoinfalse", false},
				},
			},
		},
		{
			name: "params entries added to existing",
			oldState: &GenesisState{
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"movecointrue", true},
						{"movecoinfalse", false},
					},
				},
				SendEnabled: []SendEnabled{
					{"staycoin", true},
				},
			},
			newState: &GenesisState{
				Params: Params{},
				SendEnabled: []SendEnabled{
					{"staycoin", true},
					{"movecointrue", true},
					{"movecoinfalse", false},
				},
			},
		},
		{
			name: "conflicting params ignored",
			oldState: &GenesisState{
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"staycoin", false},
					},
				},
				SendEnabled: []SendEnabled{
					{"staycoin", true},
				},
			},
			newState: &GenesisState{
				Params: Params{},
				SendEnabled: []SendEnabled{
					{"staycoin", true},
				},
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(tt *testing.T) {
			tc.oldState.MigrateSendEnabled()
			assert.Equal(tt, tc.newState, tc.oldState)
		})
	}
}

func TestGetAllSendEnabled(t *testing.T) {
	tests := []struct {
		name string
		gs   GenesisState
		exp  []SendEnabled
	}{
		{
			name: "nil and nil",
			gs: GenesisState{
				SendEnabled: nil,
				Params: Params{
					SendEnabled: nil,
				},
			},
			exp: nil,
		},
		{
			name: "nil and empty",
			gs: GenesisState{
				SendEnabled: nil,
				Params: Params{
					SendEnabled: []*SendEnabled{},
				},
			},
			exp: nil,
		},
		{
			name: "empty and nil",
			gs: GenesisState{
				SendEnabled: []SendEnabled{},
				Params: Params{
					SendEnabled: nil,
				},
			},
			exp: []SendEnabled{},
		},
		{
			name: "empty and empty",
			gs: GenesisState{
				SendEnabled: []SendEnabled{},
				Params: Params{
					SendEnabled: []*SendEnabled{},
				},
			},
			exp: []SendEnabled{},
		},
		{
			name: "one and nil",
			gs: GenesisState{
				SendEnabled: []SendEnabled{{"onenilcoin", true}},
				Params: Params{
					SendEnabled: nil,
				},
			},
			exp: []SendEnabled{{"onenilcoin", true}},
		},
		{
			name: "one and empty",
			gs: GenesisState{
				SendEnabled: []SendEnabled{{"oneemptycoin", true}},
				Params: Params{
					SendEnabled: []*SendEnabled{},
				},
			},
			exp: []SendEnabled{{"oneemptycoin", true}},
		},
		{
			name: "nil and one",
			gs: GenesisState{
				SendEnabled: nil,
				Params: Params{
					SendEnabled: []*SendEnabled{{"nilonecoin", true}},
				},
			},
			exp: []SendEnabled{{"nilonecoin", true}},
		},
		{
			name: "empty and one",
			gs: GenesisState{
				SendEnabled: []SendEnabled{},
				Params: Params{
					SendEnabled: []*SendEnabled{{"emptyonecoin", true}},
				},
			},
			exp: []SendEnabled{{"emptyonecoin", true}},
		},
		{
			name: "one and one different denoms",
			gs: GenesisState{
				SendEnabled: []SendEnabled{{"oneonediff1coin", true}},
				Params: Params{
					SendEnabled: []*SendEnabled{{"oneonediff2coin", false}},
				},
			},
			exp: []SendEnabled{{"oneonediff1coin", true}, {"oneonediff2coin", false}},
		},
		{
			name: "one and one same denoms true",
			gs: GenesisState{
				SendEnabled: []SendEnabled{{"oneonesametruecoin", true}},
				Params: Params{
					SendEnabled: []*SendEnabled{{"oneonesametruecoin", false}},
				},
			},
			exp: []SendEnabled{{"oneonesametruecoin", true}},
		},
		{
			name: "one and one same denoms false",
			gs: GenesisState{
				SendEnabled: []SendEnabled{{"oneonesamefalsecoin", false}},
				Params: Params{
					SendEnabled: []*SendEnabled{{"oneonesamefalsecoin", true}},
				},
			},
			exp: []SendEnabled{{"oneonesamefalsecoin", false}},
		},
		{
			name: "three and three with one same denom",
			gs: GenesisState{
				SendEnabled: []SendEnabled{
					{"threethree1coin", true},
					{"threethree2coin", true},
					{"threethree3coin", true},
				},
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"threethree4coin", true},
						{"threethree2coin", false},
						{"threethree5coin", true},
					},
				},
			},
			exp: []SendEnabled{
				{"threethree1coin", true},
				{"threethree2coin", true},
				{"threethree3coin", true},
				{"threethree4coin", true},
				{"threethree5coin", true},
			},
		},
		{
			name: "three and three all overlap",
			gs: GenesisState{
				SendEnabled: []SendEnabled{
					{"threethreedup1coin", false},
					{"threethreedup2coin", true},
					{"threethreedup3coin", false},
				},
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"threethreedup1coin", true},
						{"threethreedup2coin", false},
						{"threethreedup3coin", true},
					},
				},
			},
			exp: []SendEnabled{
				{"threethreedup1coin", false},
				{"threethreedup2coin", true},
				{"threethreedup3coin", false},
			},
		},
		{
			name: "three and three no overlap",
			gs: GenesisState{
				SendEnabled: []SendEnabled{
					{"threethreediff1coin", true},
					{"threethreediff2coin", false},
					{"threethreediff3coin", true},
				},
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"threethreediff4coin", false},
						{"threethreediff5coin", true},
						{"threethreediff6coin", false},
					},
				},
			},
			exp: []SendEnabled{
				{"threethreediff1coin", true},
				{"threethreediff2coin", false},
				{"threethreediff3coin", true},
				{"threethreediff4coin", false},
				{"threethreediff5coin", true},
				{"threethreediff6coin", false},
			},
		},
		{
			name: "one and three with overlap",
			gs: GenesisState{
				SendEnabled: []SendEnabled{
					{"onethreeover1coin", false},
				},
				Params: Params{
					SendEnabled: []*SendEnabled{
						{"onethreeover1coin", true},
						{"onethreeover2coin", true},
						{"onethreeover3coin", false},
					},
				},
			},
			exp: []SendEnabled{
				{"onethreeover1coin", false},
				{"onethreeover2coin", true},
				{"onethreeover3coin", false},
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(tt *testing.T) {
			actual := tc.gs.GetAllSendEnabled()
			assert.Equal(tt, tc.exp, actual)
		})
	}
}
