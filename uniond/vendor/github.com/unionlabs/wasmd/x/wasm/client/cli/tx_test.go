package cli

import (
	"encoding/hex"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/ioutils"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestParseVerificationFlags(t *testing.T) {
	mySender := sdk.MustAccAddressFromBech32("cosmos1wyqh3n50ecatjg4vww5crmtd0nmyzusnwckw4at4gluc0m5m477q4arfek")

	specs := map[string]struct {
		srcPath     string
		args        []string
		expErr      bool
		expSource   string
		expBuilder  string
		expCodeHash string
	}{
		"gov store zipped": {
			srcPath: "../../keeper/testdata/hackatom.wasm.gzip",
			args: []string{
				"--instantiate-everybody=true", "--code-hash=" + testdata.ChecksumHackatom,
				"--code-source-url=https://example.com", "--builder=cosmwasm/workspace-optimizer:0.12.11",
			},
			expBuilder:  "cosmwasm/workspace-optimizer:0.12.11",
			expSource:   "https://example.com",
			expCodeHash: testdata.ChecksumHackatom,
		},
		"gov store raw": {
			srcPath: "../../keeper/testdata/hackatom.wasm",
			args: []string{
				"--instantiate-everybody=true", "--code-hash=" + testdata.ChecksumHackatom,
				"--code-source-url=https://example.com", "--builder=cosmwasm/workspace-optimizer:0.12.11",
			},
			expBuilder:  "cosmwasm/workspace-optimizer:0.12.11",
			expSource:   "https://example.com",
			expCodeHash: testdata.ChecksumHackatom,
		},
		"gov store checksum mismatch": {
			srcPath: "../../keeper/testdata/hackatom.wasm",
			args: []string{
				"--instantiate-everybody=true", "--code-hash=0000de5e9b93b52e514c74ce87ccddb594b9bcd33b7f1af1bb6da63fc883917b",
				"--code-source-url=https://example.com", "--builder=cosmwasm/workspace-optimizer:0.12.11",
			},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			flagSet := ProposalStoreAndInstantiateContractCmd().Flags()
			require.NoError(t, flagSet.Parse(spec.args))

			gotMsg, err := parseStoreCodeArgs(spec.srcPath, mySender.String(), flagSet)
			require.NoError(t, err)
			require.True(t, ioutils.IsGzip(gotMsg.WASMByteCode))

			gotSource, gotBuilder, gotCodeHash, gotErr := parseVerificationFlags(gotMsg.WASMByteCode, flagSet)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expSource, gotSource)
			assert.Equal(t, spec.expBuilder, gotBuilder)
			assert.Equal(t, spec.expCodeHash, hex.EncodeToString(gotCodeHash))
		})
	}
}

func TestParseAccessConfigFlags(t *testing.T) {
	specs := map[string]struct {
		args   []string
		expCfg *types.AccessConfig
		expErr bool
	}{
		"nobody": {
			args:   []string{"--instantiate-nobody=true"},
			expCfg: &types.AccessConfig{Permission: types.AccessTypeNobody},
		},
		"everybody": {
			args:   []string{"--instantiate-everybody=true"},
			expCfg: &types.AccessConfig{Permission: types.AccessTypeEverybody},
		},
		"only address": {
			args:   []string{"--instantiate-only-address=cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x"},
			expErr: true,
		},
		"only address - invalid": {
			args:   []string{"--instantiate-only-address=foo"},
			expErr: true,
		},
		"any of address": {
			args:   []string{"--instantiate-anyof-addresses=cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x,cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"},
			expCfg: &types.AccessConfig{Permission: types.AccessTypeAnyOfAddresses, Addresses: []string{"cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x", "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"}},
		},
		"any of address - invalid": {
			args:   []string{"--instantiate-anyof-addresses=cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x,foo"},
			expErr: true,
		},
		"not set": {
			args: []string{},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			flags := StoreCodeCmd().Flags()
			require.NoError(t, flags.Parse(spec.args))
			gotCfg, gotErr := parseAccessConfigFlags(flags)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expCfg, gotCfg)
		})
	}
}

func TestParseStoreCodeGrants(t *testing.T) {
	specs := map[string]struct {
		src    []string
		exp    []types.CodeGrant
		expErr bool
	}{
		"wildcard : nobody": {
			src: []string{"*:nobody"},
			exp: []types.CodeGrant{{
				CodeHash:              []byte("*"),
				InstantiatePermission: &types.AccessConfig{Permission: types.AccessTypeNobody},
			}},
		},
		"wildcard : wildcard": {
			src: []string{"*:*"},
			exp: []types.CodeGrant{{
				CodeHash: []byte("*"),
			}},
		},
		"wildcard : everybody": {
			src: []string{"*:everybody"},
			exp: []types.CodeGrant{{
				CodeHash:              []byte("*"),
				InstantiatePermission: &types.AccessConfig{Permission: types.AccessTypeEverybody},
			}},
		},
		"wildcard : any of addresses - single": {
			src: []string{"*:cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x"},
			exp: []types.CodeGrant{
				{
					CodeHash: []byte("*"),
					InstantiatePermission: &types.AccessConfig{
						Permission: types.AccessTypeAnyOfAddresses,
						Addresses:  []string{"cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x"},
					},
				},
			},
		},
		"wildcard : any of addresses - multiple": {
			src: []string{"*:cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x,cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"},
			exp: []types.CodeGrant{
				{
					CodeHash: []byte("*"),
					InstantiatePermission: &types.AccessConfig{
						Permission: types.AccessTypeAnyOfAddresses,
						Addresses:  []string{"cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x", "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"},
					},
				},
			},
		},
		"multiple code hashes with different permissions": {
			src: []string{"any_checksum_1:cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x,cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", "any_checksum_2:nobody"},
			exp: []types.CodeGrant{
				{
					CodeHash: []byte("any_checksum_1"),
					InstantiatePermission: &types.AccessConfig{
						Permission: types.AccessTypeAnyOfAddresses,
						Addresses:  []string{"cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x", "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"},
					},
				}, {
					CodeHash: []byte("any_checksum_2"),
					InstantiatePermission: &types.AccessConfig{
						Permission: types.AccessTypeNobody,
					},
				},
			},
		},
		"code hash : wildcard": {
			src: []string{"any_checksum_1:*"},
			exp: []types.CodeGrant{{
				CodeHash: []byte("any_checksum_1"),
			}},
		},
		"code hash : any of addresses - empty list": {
			src:    []string{"any_checksum_1:"},
			expErr: true,
		},
		"code hash : any of addresses - invalid address": {
			src:    []string{"any_checksum_1:foo"},
			expErr: true,
		},
		"code hash : any of addresses - duplicate address": {
			src:    []string{"any_checksum_1:cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x,cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x"},
			expErr: true,
		},
		"empty code hash": {
			src:    []string{":everyone"},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			got, gotErr := parseStoreCodeGrants(spec.src)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.exp, got)
		})
	}
}
