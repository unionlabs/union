package types

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v2"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"cosmossdk.io/x/gov/types/v1beta1"
)

func TestValidateProposalCommons(t *testing.T) {
	type commonProposal struct {
		Title, Description string
	}

	specs := map[string]struct {
		src    commonProposal
		expErr bool
	}{
		"all good": {src: commonProposal{
			Title:       "Foo",
			Description: "Bar",
		}},
		"prevent empty title": {
			src: commonProposal{
				Description: "Bar",
			},
			expErr: true,
		},
		"prevent white space only title": {
			src: commonProposal{
				Title:       " ",
				Description: "Bar",
			},
			expErr: true,
		},
		"prevent leading white spaces in title": {
			src: commonProposal{
				Title:       " Foo",
				Description: "Bar",
			},
			expErr: true,
		},
		"prevent title exceeds max length ": {
			src: commonProposal{
				Title:       strings.Repeat("a", v1beta1.MaxTitleLength+1),
				Description: "Bar",
			},
			expErr: true,
		},
		"prevent empty description": {
			src: commonProposal{
				Title: "Foo",
			},
			expErr: true,
		},
		"prevent leading white spaces in description": {
			src: commonProposal{
				Title:       "Foo",
				Description: " Bar",
			},
			expErr: true,
		},
		"prevent white space only description": {
			src: commonProposal{
				Title:       "Foo",
				Description: " ",
			},
			expErr: true,
		},
		"prevent descr exceeds max length ": {
			src: commonProposal{
				Title:       "Foo",
				Description: strings.Repeat("a", v1beta1.MaxDescriptionLength+1),
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := validateProposalCommons(spec.src.Title, spec.src.Description)
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateStoreCodeProposal(t *testing.T) {
	var anyAddress sdk.AccAddress = bytes.Repeat([]byte{0x0}, ContractAddrLen)

	specs := map[string]struct {
		src    *StoreCodeProposal
		expErr bool
	}{
		"all good": {
			src: StoreCodeProposalFixture(),
		},
		"all good no code verification info": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.Source = ""
				p.Builder = ""
				p.CodeHash = nil
			}),
		},
		"source missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.Source = ""
			}),
			expErr: true,
		},
		"builder missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.Builder = ""
			}),
			expErr: true,
		},
		"code hash missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.CodeHash = nil
			}),
			expErr: true,
		},
		"with instantiate permission": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				accessConfig := AccessTypeAnyOfAddresses.With(anyAddress)
				p.InstantiatePermission = &accessConfig
			}),
		},
		"base data missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"run_as missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.RunAs = ""
			}),
			expErr: true,
		},
		"run_as invalid": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.RunAs = invalidAddress
			}),
			expErr: true,
		},
		"wasm code missing": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.WASMByteCode = nil
			}),
			expErr: true,
		},
		"wasm code invalid": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.WASMByteCode = bytes.Repeat([]byte{0x0}, MaxProposalWasmSize+1)
			}),
			expErr: true,
		},
		"with invalid instantiate permission": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.InstantiatePermission = &AccessConfig{}
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateInstantiateContractProposal(t *testing.T) {
	specs := map[string]struct {
		src    *InstantiateContractProposal
		expErr bool
	}{
		"all good": {
			src: InstantiateContractProposalFixture(),
		},
		"without admin": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Admin = ""
			}),
		},
		"without init msg": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"with invalid init msg": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Msg = []byte("not a json string")
			}),
			expErr: true,
		},
		"without init funds": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Funds = nil
			}),
		},
		"base data missing": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"run_as missing": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.RunAs = ""
			}),
			expErr: true,
		},
		"run_as invalid": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.RunAs = invalidAddress
			}),
			expErr: true,
		},
		"admin invalid": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Admin = invalidAddress
			}),
			expErr: true,
		},
		"code id empty": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.CodeID = 0
			}),
			expErr: true,
		},
		"label empty": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Label = ""
			}),
			expErr: true,
		},
		"init funds negative": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(-1)}}
			}),
			expErr: true,
		},
		"init funds with duplicates": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(1)}, {Denom: "foo", Amount: sdkmath.NewInt(2)}}
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateInstantiateContract2Proposal(t *testing.T) {
	specs := map[string]struct {
		src    *InstantiateContract2Proposal
		expErr bool
	}{
		"all good": {
			src: InstantiateContract2ProposalFixture(),
		},
		"without admin": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Admin = ""
			}),
		},
		"without init msg": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"with invalid init msg": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Msg = []byte("not a json string")
			}),
			expErr: true,
		},
		"without init funds": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Funds = nil
			}),
		},
		"base data missing": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"run_as missing": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.RunAs = ""
			}),
			expErr: true,
		},
		"run_as invalid": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.RunAs = invalidAddress
			}),
			expErr: true,
		},
		"admin invalid": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Admin = invalidAddress
			}),
			expErr: true,
		},
		"code id empty": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.CodeID = 0
			}),
			expErr: true,
		},
		"label empty": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Label = ""
			}),
			expErr: true,
		},
		"untrimmed label ": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Label = "    label   "
			}),
			expErr: true,
		},
		"init funds negative": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(-1)}}
			}),
			expErr: true,
		},
		"init funds with duplicates": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(1)}, {Denom: "foo", Amount: sdkmath.NewInt(2)}}
			}),
			expErr: true,
		},
		"init with empty salt": {
			src: InstantiateContract2ProposalFixture(func(p *InstantiateContract2Proposal) {
				p.Salt = nil
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateStoreAndInstantiateContractProposal(t *testing.T) {
	var anyAddress sdk.AccAddress = bytes.Repeat([]byte{0x0}, ContractAddrLen)

	specs := map[string]struct {
		src    *StoreAndInstantiateContractProposal
		expErr bool
	}{
		"all good": {
			src: StoreAndInstantiateContractProposalFixture(),
		},
		"all good no code verification info": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Source = ""
				p.Builder = ""
				p.CodeHash = nil
			}),
		},
		"source missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Source = ""
			}),
			expErr: true,
		},
		"builder missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Builder = ""
			}),
			expErr: true,
		},
		"code hash missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.CodeHash = nil
			}),
			expErr: true,
		},
		"with instantiate permission": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				accessConfig := AccessTypeAnyOfAddresses.With(anyAddress)
				p.InstantiatePermission = &accessConfig
			}),
		},
		"base data missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"run_as missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.RunAs = ""
			}),
			expErr: true,
		},
		"run_as invalid": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.RunAs = invalidAddress
			}),
			expErr: true,
		},
		"wasm code missing": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.WASMByteCode = nil
			}),
			expErr: true,
		},
		"wasm code invalid": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.WASMByteCode = bytes.Repeat([]byte{0x0}, MaxProposalWasmSize+1)
			}),
			expErr: true,
		},
		"with invalid instantiate permission": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.InstantiatePermission = &AccessConfig{}
			}),
			expErr: true,
		},
		"without admin": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Admin = ""
			}),
		},
		"without init msg": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"with invalid init msg": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Msg = []byte("not a json string")
			}),
			expErr: true,
		},
		"without init funds": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Funds = nil
			}),
		},
		"admin invalid": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Admin = invalidAddress
			}),
			expErr: true,
		},
		"label empty": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Label = ""
			}),
			expErr: true,
		},
		"init funds negative": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(-1)}}
			}),
			expErr: true,
		},
		"init funds with duplicates": {
			src: StoreAndInstantiateContractProposalFixture(func(p *StoreAndInstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(1)}, {Denom: "foo", Amount: sdkmath.NewInt(2)}}
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateMigrateContractProposal(t *testing.T) {
	invalidAddress := "invalid address2"

	specs := map[string]struct {
		src    *MigrateContractProposal
		expErr bool
	}{
		"all good": {
			src: MigrateContractProposalFixture(),
		},
		"without migrate msg": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"migrate msg with invalid json": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.Msg = []byte("not a json message")
			}),
			expErr: true,
		},
		"base data missing": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"contract missing": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.Contract = ""
			}),
			expErr: true,
		},
		"contract invalid": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.Contract = invalidAddress
			}),
			expErr: true,
		},
		"code id empty": {
			src: MigrateContractProposalFixture(func(p *MigrateContractProposal) {
				p.CodeID = 0
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateSudoContractProposal(t *testing.T) {
	specs := map[string]struct {
		src    *SudoContractProposal
		expErr bool
	}{
		"all good": {
			src: SudoContractProposalFixture(),
		},
		"msg is nil": {
			src: SudoContractProposalFixture(func(p *SudoContractProposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"msg with invalid json": {
			src: SudoContractProposalFixture(func(p *SudoContractProposal) {
				p.Msg = []byte("not a json message")
			}),
			expErr: true,
		},
		"base data missing": {
			src: SudoContractProposalFixture(func(p *SudoContractProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"contract missing": {
			src: SudoContractProposalFixture(func(p *SudoContractProposal) {
				p.Contract = ""
			}),
			expErr: true,
		},
		"contract invalid": {
			src: SudoContractProposalFixture(func(p *SudoContractProposal) {
				p.Contract = invalidAddress
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateExecuteContractProposal(t *testing.T) {
	specs := map[string]struct {
		src    *ExecuteContractProposal
		expErr bool
	}{
		"all good": {
			src: ExecuteContractProposalFixture(),
		},
		"msg is nil": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.Msg = nil
			}),
			expErr: true,
		},
		"msg with invalid json": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.Msg = []byte("not a valid json message")
			}),
			expErr: true,
		},
		"base data missing": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"contract missing": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.Contract = ""
			}),
			expErr: true,
		},
		"contract invalid": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.Contract = invalidAddress
			}),
			expErr: true,
		},
		"run as is invalid": {
			src: ExecuteContractProposalFixture(func(p *ExecuteContractProposal) {
				p.RunAs = invalidAddress
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateUpdateAdminProposal(t *testing.T) {
	specs := map[string]struct {
		src    *UpdateAdminProposal
		expErr bool
	}{
		"all good": {
			src: UpdateAdminProposalFixture(),
		},
		"base data missing": {
			src: UpdateAdminProposalFixture(func(p *UpdateAdminProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"contract missing": {
			src: UpdateAdminProposalFixture(func(p *UpdateAdminProposal) {
				p.Contract = ""
			}),
			expErr: true,
		},
		"contract invalid": {
			src: UpdateAdminProposalFixture(func(p *UpdateAdminProposal) {
				p.Contract = invalidAddress
			}),
			expErr: true,
		},
		"admin missing": {
			src: UpdateAdminProposalFixture(func(p *UpdateAdminProposal) {
				p.NewAdmin = ""
			}),
			expErr: true,
		},
		"admin invalid": {
			src: UpdateAdminProposalFixture(func(p *UpdateAdminProposal) {
				p.NewAdmin = invalidAddress
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateClearAdminProposal(t *testing.T) {
	specs := map[string]struct {
		src    *ClearAdminProposal
		expErr bool
	}{
		"all good": {
			src: ClearAdminProposalFixture(),
		},
		"base data missing": {
			src: ClearAdminProposalFixture(func(p *ClearAdminProposal) {
				p.Title = ""
			}),
			expErr: true,
		},
		"contract missing": {
			src: ClearAdminProposalFixture(func(p *ClearAdminProposal) {
				p.Contract = ""
			}),
			expErr: true,
		},
		"contract invalid": {
			src: ClearAdminProposalFixture(func(p *ClearAdminProposal) {
				p.Contract = invalidAddress
			}),
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestProposalStrings(t *testing.T) {
	specs := map[string]struct {
		src v1beta1.Content
		exp string
	}{
		"store code": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.WASMByteCode = []byte{0o1, 0o2, 0o3, 0o4, 0o5, 0o6, 0o7, 0x08, 0x09, 0x0a}
			}),
			exp: `Store Code Proposal:
  Title:       Foo
  Description: Bar
  Run as:      cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  WasmCode:    0102030405060708090A
  Source:      https://example.com/
  Builder:     cosmwasm/workspace-optimizer:v0.12.8
  Code Hash:   6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D
`,
		},
		"instantiate contract": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(1)}, {Denom: "bar", Amount: sdkmath.NewInt(2)}}
			}),
			exp: `Instantiate Code Proposal:
  Title:       Foo
  Description: Bar
  Run as:      cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  Admin:       cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  Code id:     1
  Label:       testing
  Msg:         "{\"verifier\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\",\"beneficiary\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\"}"
  Funds:       1foo,2bar
`,
		},
		"instantiate contract without funds": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) { p.Funds = nil }),
			exp: `Instantiate Code Proposal:
  Title:       Foo
  Description: Bar
  Run as:      cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  Admin:       cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  Code id:     1
  Label:       testing
  Msg:         "{\"verifier\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\",\"beneficiary\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\"}"
  Funds:       
`,
		},
		"instantiate contract without admin": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) { p.Admin = "" }),
			exp: `Instantiate Code Proposal:
  Title:       Foo
  Description: Bar
  Run as:      cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
  Admin:       
  Code id:     1
  Label:       testing
  Msg:         "{\"verifier\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\",\"beneficiary\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\"}"
  Funds:       
`,
		},
		"migrate contract": {
			src: MigrateContractProposalFixture(),
			exp: `Migrate Contract Proposal:
  Title:       Foo
  Description: Bar
  Contract:    cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
  Code id:     1
  Msg:         "{\"verifier\":\"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4\"}"
`,
		},
		"update admin": {
			src: UpdateAdminProposalFixture(),
			exp: `Update Contract Admin Proposal:
  Title:       Foo
  Description: Bar
  Contract:    cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
  New Admin:   cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
`,
		},
		"clear admin": {
			src: ClearAdminProposalFixture(),
			exp: `Clear Contract Admin Proposal:
  Title:       Foo
  Description: Bar
  Contract:    cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
`,
		},
		"pin codes": {
			src: &PinCodesProposal{
				Title:       "Foo",
				Description: "Bar",
				CodeIDs:     []uint64{1, 2, 3},
			},
			exp: `Pin Wasm Codes Proposal:
  Title:       Foo
  Description: Bar
  Codes:       [1 2 3]
`,
		},
		"unpin codes": {
			src: &UnpinCodesProposal{
				Title:       "Foo",
				Description: "Bar",
				CodeIDs:     []uint64{3, 2, 1},
			},
			exp: `Unpin Wasm Codes Proposal:
  Title:       Foo
  Description: Bar
  Codes:       [3 2 1]
`,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			assert.Equal(t, spec.exp, spec.src.String())
		})
	}
}

func TestProposalYaml(t *testing.T) {
	specs := map[string]struct {
		src v1beta1.Content
		exp string
	}{
		"store code": {
			src: StoreCodeProposalFixture(func(p *StoreCodeProposal) {
				p.WASMByteCode = []byte{0o1, 0o2, 0o3, 0o4, 0o5, 0o6, 0o7, 0x08, 0x09, 0x0a}
			}),
			exp: `title: Foo
description: Bar
run_as: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
wasm_byte_code: AQIDBAUGBwgJCg==
instantiate_permission: null
source: https://example.com/
builder: cosmwasm/workspace-optimizer:v0.12.8
code_hash: 6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d
`,
		},
		"instantiate contract": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) {
				p.Funds = sdk.Coins{{Denom: "foo", Amount: sdkmath.NewInt(1)}, {Denom: "bar", Amount: sdkmath.NewInt(2)}}
			}),
			exp: `title: Foo
description: Bar
run_as: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
admin: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
code_id: 1
label: testing
msg: '{"verifier":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4","beneficiary":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"}'
funds:
- denom: foo
  amount: "1"
- denom: bar
  amount: "2"
`,
		},
		"instantiate contract without funds": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) { p.Funds = nil }),
			exp: `title: Foo
description: Bar
run_as: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
admin: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
code_id: 1
label: testing
msg: '{"verifier":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4","beneficiary":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"}'
funds: []
`,
		},
		"instantiate contract without admin": {
			src: InstantiateContractProposalFixture(func(p *InstantiateContractProposal) { p.Admin = "" }),
			exp: `title: Foo
description: Bar
run_as: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
admin: ""
code_id: 1
label: testing
msg: '{"verifier":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4","beneficiary":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"}'
funds: []
`,
		},
		"migrate contract": {
			src: MigrateContractProposalFixture(),
			exp: `title: Foo
description: Bar
contract: cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
code_id: 1
msg: '{"verifier":"cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"}'
`,
		},
		"update admin": {
			src: UpdateAdminProposalFixture(),
			exp: `title: Foo
description: Bar
new_admin: cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4
contract: cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
`,
		},
		"clear admin": {
			src: ClearAdminProposalFixture(),
			exp: `title: Foo
description: Bar
contract: cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr
`,
		},
		"pin codes": {
			src: &PinCodesProposal{
				Title:       "Foo",
				Description: "Bar",
				CodeIDs:     []uint64{1, 2, 3},
			},
			exp: `title: Foo
description: Bar
code_ids:
- 1
- 2
- 3
`,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			v, err := yaml.Marshal(&spec.src) //nolint:gosec
			require.NoError(t, err)
			assert.Equal(t, spec.exp, string(v))
		})
	}
}

func TestUnmarshalContentFromJson(t *testing.T) {
	specs := map[string]struct {
		src string
		got v1beta1.Content
		exp v1beta1.Content
	}{
		"instantiate ": {
			src: `
{
	"title": "foo",
	"description": "bar",
	"admin": "myAdminAddress",
	"code_id": 1,
	"funds": [{"denom": "ALX", "amount": "2"},{"denom": "BLX","amount": "3"}],
	"msg": {},
	"label": "testing",
	"run_as": "myRunAsAddress"
}`,
			got: &InstantiateContractProposal{},
			exp: &InstantiateContractProposal{
				Title:       "foo",
				Description: "bar",
				RunAs:       "myRunAsAddress",
				Admin:       "myAdminAddress",
				CodeID:      1,
				Label:       "testing",
				Msg:         []byte("{}"),
				Funds:       sdk.NewCoins(sdk.NewCoin("ALX", sdkmath.NewInt(2)), sdk.NewCoin("BLX", sdkmath.NewInt(3))),
			},
		},
		"migrate ": {
			src: `
{
	"title": "foo",
	"description": "bar",
	"code_id": 1,
	"contract": "myContractAddr",
	"msg": {},
	"run_as": "myRunAsAddress"
}`,
			got: &MigrateContractProposal{},
			exp: &MigrateContractProposal{
				Title:       "foo",
				Description: "bar",
				Contract:    "myContractAddr",
				CodeID:      1,
				Msg:         []byte("{}"),
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			require.NoError(t, json.Unmarshal([]byte(spec.src), spec.got))
			assert.Equal(t, spec.exp, spec.got)
		})
	}
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func StoreCodeProposalFixture(mutators ...func(*StoreCodeProposal)) *StoreCodeProposal {
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	wasm := []byte{0x0}
	// got the value from shell sha256sum
	codeHash, err := hex.DecodeString("6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D")
	if err != nil {
		panic(err)
	}

	p := &StoreCodeProposal{
		Title:        "Foo",
		Description:  "Bar",
		RunAs:        anyAddress,
		WASMByteCode: wasm,
		Source:       "https://example.com/",
		Builder:      "cosmwasm/workspace-optimizer:v0.12.8",
		CodeHash:     codeHash,
	}
	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func InstantiateContractProposalFixture(mutators ...func(p *InstantiateContractProposal)) *InstantiateContractProposal {
	var (
		anyValidAddress sdk.AccAddress = bytes.Repeat([]byte{0x1}, ContractAddrLen)

		initMsg = struct {
			Verifier    sdk.AccAddress `json:"verifier"`
			Beneficiary sdk.AccAddress `json:"beneficiary"`
		}{
			Verifier:    anyValidAddress,
			Beneficiary: anyValidAddress,
		}
	)
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"

	initMsgBz, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}
	p := &InstantiateContractProposal{
		Title:       "Foo",
		Description: "Bar",
		RunAs:       anyAddress,
		Admin:       anyAddress,
		CodeID:      1,
		Label:       "testing",
		Msg:         initMsgBz,
		Funds:       nil,
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func InstantiateContract2ProposalFixture(mutators ...func(p *InstantiateContract2Proposal)) *InstantiateContract2Proposal {
	var (
		anyValidAddress sdk.AccAddress = bytes.Repeat([]byte{0x1}, ContractAddrLen)

		initMsg = struct {
			Verifier    sdk.AccAddress `json:"verifier"`
			Beneficiary sdk.AccAddress `json:"beneficiary"`
		}{
			Verifier:    anyValidAddress,
			Beneficiary: anyValidAddress,
		}
	)
	const (
		anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
		mySalt     = "myDefaultSalt"
	)

	initMsgBz, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}
	p := &InstantiateContract2Proposal{
		Title:       "Foo",
		Description: "Bar",
		RunAs:       anyAddress,
		Admin:       anyAddress,
		CodeID:      1,
		Label:       "testing",
		Msg:         initMsgBz,
		Funds:       nil,
		Salt:        []byte(mySalt),
		FixMsg:      false,
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func StoreAndInstantiateContractProposalFixture(mutators ...func(p *StoreAndInstantiateContractProposal)) *StoreAndInstantiateContractProposal {
	var (
		anyValidAddress sdk.AccAddress = bytes.Repeat([]byte{0x1}, ContractAddrLen)

		initMsg = struct {
			Verifier    sdk.AccAddress `json:"verifier"`
			Beneficiary sdk.AccAddress `json:"beneficiary"`
		}{
			Verifier:    anyValidAddress,
			Beneficiary: anyValidAddress,
		}
	)
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	wasm := []byte{0x0}
	// got the value from shell sha256sum
	codeHash, err := hex.DecodeString("6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D")
	if err != nil {
		panic(err)
	}

	initMsgBz, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}
	p := &StoreAndInstantiateContractProposal{
		Title:        "Foo",
		Description:  "Bar",
		RunAs:        anyAddress,
		WASMByteCode: wasm,
		Source:       "https://example.com/",
		Builder:      "cosmwasm/workspace-optimizer:v0.12.9",
		CodeHash:     codeHash,
		Admin:        anyAddress,
		Label:        "testing",
		Msg:          initMsgBz,
		Funds:        nil,
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func MigrateContractProposalFixture(mutators ...func(p *MigrateContractProposal)) *MigrateContractProposal {
	var (
		anyValidAddress sdk.AccAddress = bytes.Repeat([]byte{0x1}, ContractAddrLen)

		migMsg = struct {
			Verifier sdk.AccAddress `json:"verifier"`
		}{Verifier: anyValidAddress}
	)

	migMsgBz, err := json.Marshal(migMsg)
	if err != nil {
		panic(err)
	}
	const (
		contractAddr = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
		anyAddress   = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	)
	p := &MigrateContractProposal{
		Title:       "Foo",
		Description: "Bar",
		Contract:    contractAddr,
		CodeID:      1,
		Msg:         migMsgBz,
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func SudoContractProposalFixture(mutators ...func(p *SudoContractProposal)) *SudoContractProposal {
	const (
		contractAddr = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
	)

	p := &SudoContractProposal{
		Title:       "Foo",
		Description: "Bar",
		Contract:    contractAddr,
		Msg:         []byte(`{"do":"something"}`),
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func ExecuteContractProposalFixture(mutators ...func(p *ExecuteContractProposal)) *ExecuteContractProposal {
	const (
		contractAddr = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
		anyAddress   = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	)

	p := &ExecuteContractProposal{
		Title:       "Foo",
		Description: "Bar",
		Contract:    contractAddr,
		RunAs:       anyAddress,
		Msg:         []byte(`{"do":"something"}`),
		Funds: sdk.Coins{{
			Denom:  "stake",
			Amount: sdkmath.NewInt(1),
		}},
	}

	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func UpdateAdminProposalFixture(mutators ...func(p *UpdateAdminProposal)) *UpdateAdminProposal {
	const (
		contractAddr = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
		anyAddress   = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	)

	p := &UpdateAdminProposal{
		Title:       "Foo",
		Description: "Bar",
		NewAdmin:    anyAddress,
		Contract:    contractAddr,
	}
	for _, m := range mutators {
		m(p)
	}
	return p
}

// Deprecated: all gov v1beta1 types are supported for gov store only
func ClearAdminProposalFixture(mutators ...func(p *ClearAdminProposal)) *ClearAdminProposal {
	const contractAddr = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
	p := &ClearAdminProposal{
		Title:       "Foo",
		Description: "Bar",
		Contract:    contractAddr,
	}
	for _, m := range mutators {
		m(p)
	}
	return p
}
