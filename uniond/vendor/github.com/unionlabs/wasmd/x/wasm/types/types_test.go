package types

import (
	"bytes"
	"context"
	"strings"
	"testing"
	"time"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"cosmossdk.io/math/unsafe"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"cosmossdk.io/x/gov/types/v1beta1"
)

func TestContractInfoValidateBasic(t *testing.T) {
	specs := map[string]struct {
		srcMutator func(*ContractInfo)
		expError   bool
	}{
		"all good": {srcMutator: func(_ *ContractInfo) {}},
		"code id empty": {
			srcMutator: func(c *ContractInfo) { c.CodeID = 0 },
			expError:   true,
		},
		"creator empty": {
			srcMutator: func(c *ContractInfo) { c.Creator = "" },
			expError:   true,
		},
		"creator not an address": {
			srcMutator: func(c *ContractInfo) { c.Creator = invalidAddress },
			expError:   true,
		},
		"admin empty": {
			srcMutator: func(c *ContractInfo) { c.Admin = "" },
			expError:   false,
		},
		"admin not an address": {
			srcMutator: func(c *ContractInfo) { c.Admin = invalidAddress },
			expError:   true,
		},
		"label empty": {
			srcMutator: func(c *ContractInfo) { c.Label = "" },
			expError:   true,
		},
		"label exceeds limit": {
			srcMutator: func(c *ContractInfo) { c.Label = strings.Repeat("a", MaxLabelSize+1) },
			expError:   true,
		},
		"invalid extension": {
			srcMutator: func(c *ContractInfo) {
				// any protobuf type with ValidateBasic method
				codecAny, err := codectypes.NewAnyWithValue(&v1beta1.TextProposal{})

				require.NoError(t, err)
				c.Extension = codecAny
			},
			expError: true,
		},
		"not validatable extension": {
			srcMutator: func(c *ContractInfo) {
				// any protobuf type with ValidateBasic method
				codecAny, err := codectypes.NewAnyWithValue(&v1beta1.Proposal{})

				require.NoError(t, err)
				c.Extension = codecAny
			},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			state := ContractInfoFixture(spec.srcMutator)
			got := state.ValidateBasic()
			if spec.expError {
				require.Error(t, got)
				return
			}
			require.NoError(t, got)
		})
	}
}

func TestCodeInfoValidateBasic(t *testing.T) {
	specs := map[string]struct {
		srcMutator func(*CodeInfo)
		expError   bool
	}{
		"all good": {srcMutator: func(_ *CodeInfo) {}},
		"code hash empty": {
			srcMutator: func(c *CodeInfo) { c.CodeHash = []byte{} },
			expError:   true,
		},
		"code hash nil": {
			srcMutator: func(c *CodeInfo) { c.CodeHash = nil },
			expError:   true,
		},
		"creator empty": {
			srcMutator: func(c *CodeInfo) { c.Creator = "" },
			expError:   true,
		},
		"creator not an address": {
			srcMutator: func(c *CodeInfo) { c.Creator = invalidAddress },
			expError:   true,
		},
		"Instantiate config invalid": {
			srcMutator: func(c *CodeInfo) { c.InstantiateConfig = AccessConfig{} },
			expError:   true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			state := CodeInfoFixture(spec.srcMutator)
			got := state.ValidateBasic()
			if spec.expError {
				require.Error(t, got)
				return
			}
			require.NoError(t, got)
		})
	}
}

func TestContractInfoSetExtension(t *testing.T) {
	anyTime := time.Now().UTC()
	aNestedProtobufExt := func() ContractInfoExtension {
		// using gov proposal here as a random protobuf types as it contains an Any type inside for nested unpacking
		myExtension, err := v1beta1.NewProposal(&v1beta1.TextProposal{Title: "bar"}, 1, anyTime, anyTime)
		require.NoError(t, err)
		myExtension.TotalDeposit = nil
		return &myExtension
	}

	specs := map[string]struct {
		src    ContractInfoExtension
		expErr bool
		expNil bool
	}{
		"all good with any proto extension": {
			src: aNestedProtobufExt(),
		},
		"nil allowed": {
			src:    nil,
			expNil: true,
		},
		"validated and accepted": {
			src: &v1beta1.TextProposal{Title: "bar", Description: "set"},
		},
		"validated and rejected": {
			src:    &v1beta1.TextProposal{Title: "bar"},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var c ContractInfo
			gotErr := c.SetExtension(spec.src)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			if spec.expNil {
				return
			}
			require.NotNil(t, c.Extension)
			assert.NotNil(t, c.Extension.GetCachedValue())
		})
	}
}

func TestContractInfoMarshalUnmarshal(t *testing.T) {
	var myAddr sdk.AccAddress = rand.Bytes(ContractAddrLen)
	var myOtherAddr sdk.AccAddress = rand.Bytes(ContractAddrLen)
	anyPos := AbsoluteTxPosition{BlockHeight: 1, TxIndex: 2}

	anyTime := time.Now().UTC()
	// using gov proposal here as a random protobuf types as it contains an Any type inside for nested unpacking
	myExtension, err := v1beta1.NewProposal(&v1beta1.TextProposal{Title: "bar"}, 1, anyTime, anyTime)
	require.NoError(t, err)
	myExtension.TotalDeposit = nil

	src := NewContractInfo(1, myAddr, myOtherAddr, "bar", &anyPos)
	err = src.SetExtension(&myExtension)
	require.NoError(t, err)

	interfaceRegistry := codectypes.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	RegisterInterfaces(interfaceRegistry)
	// register proposal as extension type
	interfaceRegistry.RegisterImplementations(
		(*ContractInfoExtension)(nil),
		&v1beta1.Proposal{},
	)
	// register gov types for nested Anys
	v1beta1.RegisterInterfaces(interfaceRegistry)

	// when encode
	bz, err := marshaler.Marshal(&src)
	require.NoError(t, err)
	// and decode
	var dest ContractInfo
	err = marshaler.Unmarshal(bz, &dest)
	// then
	require.NoError(t, err)
	assert.Equal(t, src, dest)
	// and sanity check nested any
	var destExt v1beta1.Proposal
	require.NoError(t, dest.ReadExtension(&destExt))
	assert.Equal(t, destExt.GetTitle(), "bar")
}

func TestContractInfoReadExtension(t *testing.T) {
	anyTime := time.Now().UTC()
	myExtension, err := v1beta1.NewProposal(&v1beta1.TextProposal{Title: "foo"}, 1, anyTime, anyTime)
	require.NoError(t, err)
	type TestExtensionAsStruct struct {
		ContractInfoExtension
	}

	specs := map[string]struct {
		setup  func(*ContractInfo)
		param  func() ContractInfoExtension
		expVal ContractInfoExtension
		expErr bool
	}{
		"all good": {
			setup: func(i *ContractInfo) {
				err = i.SetExtension(&myExtension)
				require.NoError(t, err)
			},
			param: func() ContractInfoExtension {
				return &v1beta1.Proposal{}
			},
			expVal: &myExtension,
		},
		"no extension set": {
			setup: func(i *ContractInfo) {
			},
			param: func() ContractInfoExtension {
				return &v1beta1.Proposal{}
			},
			expVal: &v1beta1.Proposal{},
		},
		"nil argument value": {
			setup: func(i *ContractInfo) {
				err = i.SetExtension(&myExtension)
				require.NoError(t, err)
			},
			param: func() ContractInfoExtension {
				return nil
			},
			expErr: true,
		},
		"non matching types": {
			setup: func(i *ContractInfo) {
				err = i.SetExtension(&myExtension)
				require.NoError(t, err)
			},
			param: func() ContractInfoExtension {
				return &v1beta1.TextProposal{}
			},
			expErr: true,
		},
		"not a pointer argument": {
			setup: func(i *ContractInfo) {
			},
			param: func() ContractInfoExtension {
				return TestExtensionAsStruct{}
			},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var c ContractInfo
			spec.setup(&c)
			// when

			gotValue := spec.param()
			gotErr := c.ReadExtension(gotValue)

			// then
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expVal, gotValue)
		})
	}
}

func TestNewEnv(t *testing.T) {
	myTime := time.Unix(0, 1619700924259075000)
	t.Logf("++ unix: %d", myTime.UnixNano())
	var myContractAddr sdk.AccAddress = randBytes(ContractAddrLen)
	specs := map[string]struct {
		srcCtx sdk.Context
		exp    wasmvmtypes.Env
	}{
		"all good with tx counter": {
			srcCtx: WithTXCounter(sdk.Context{}.WithBlockHeight(1).WithBlockTime(myTime).WithChainID("testing").WithContext(context.Background()), 0),
			exp: wasmvmtypes.Env{
				Block: wasmvmtypes.BlockInfo{
					Height:  1,
					Time:    1619700924259075000,
					ChainID: "testing",
				},
				Contract: wasmvmtypes.ContractInfo{
					Address: myContractAddr.String(),
				},
				Transaction: &wasmvmtypes.TransactionInfo{Index: 0},
			},
		},
		"without tx counter": {
			srcCtx: sdk.Context{}.WithBlockHeight(1).WithBlockTime(myTime).WithChainID("testing").WithContext(context.Background()),
			exp: wasmvmtypes.Env{
				Block: wasmvmtypes.BlockInfo{
					Height:  1,
					Time:    1619700924259075000,
					ChainID: "testing",
				},
				Contract: wasmvmtypes.ContractInfo{
					Address: myContractAddr.String(),
				},
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			assert.Equal(t, spec.exp, NewEnv(spec.srcCtx, myContractAddr))
		})
	}
}

func TestVerifyAddressLen(t *testing.T) {
	specs := map[string]struct {
		src    []byte
		expErr bool
	}{
		"valid contract address": {
			src: bytes.Repeat([]byte{1}, 32),
		},
		"valid legacy address": {
			src: bytes.Repeat([]byte{1}, 20),
		},
		"address too short for legacy": {
			src:    bytes.Repeat([]byte{1}, 19),
			expErr: true,
		},
		"address too short for contract": {
			src:    bytes.Repeat([]byte{1}, 31),
			expErr: true,
		},
		"address too long for legacy": {
			src:    bytes.Repeat([]byte{1}, 21),
			expErr: true,
		},
		"address too long for contract": {
			src:    bytes.Repeat([]byte{1}, 33),
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotErr := VerifyAddressLen()(spec.src)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
		})
	}
}

func TestAccessConfigSubset(t *testing.T) {
	// read
	// <, <= is subset of
	// !< is not subset of
	specs := map[string]struct {
		check    AccessConfig
		superSet AccessConfig
		isSubSet bool
	}{
		// nobody
		"nobody <= nobody": {
			superSet: AccessConfig{Permission: AccessTypeNobody},
			check:    AccessConfig{Permission: AccessTypeNobody},
			isSubSet: true,
		},
		"anyOf !< nobody": {
			superSet: AccessConfig{Permission: AccessTypeNobody},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"foobar"}},
			isSubSet: false,
		},
		"everybody !< nobody ": {
			superSet: AccessConfig{Permission: AccessTypeNobody},
			check:    AccessConfig{Permission: AccessTypeEverybody},
			isSubSet: false,
		},
		"unspecified !< nobody": {
			superSet: AccessConfig{Permission: AccessTypeNobody},
			check:    AccessConfig{Permission: AccessTypeUnspecified},
			isSubSet: false,
		},
		// any of
		"nobody < anyOf": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			check:    AccessConfig{Permission: AccessTypeNobody},
			isSubSet: true,
		},
		"anyOf < anyOf": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			isSubSet: true,
		},
		"anyOf(multiple) < anyOf(multiple)": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner", "other"}},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner", "other"}},
			isSubSet: true,
		},
		"anyOf(multiple, other) !< anyOf(multiple)": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner", "other", "foo"}},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner", "other", "bar"}},
			isSubSet: false,
		},
		"anyOf(multiple) !< anyOf": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner", "other"}},
			isSubSet: false,
		},
		"everybody !< anyOf": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			check:    AccessConfig{Permission: AccessTypeEverybody},
			isSubSet: false,
		},
		"unspecified !< anyOf ": {
			superSet: AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"owner"}},
			check:    AccessConfig{Permission: AccessTypeUnspecified},
			isSubSet: false,
		},
		// everybody
		"nobody < everybody": {
			superSet: AccessConfig{Permission: AccessTypeEverybody},
			check:    AccessConfig{Permission: AccessTypeNobody},
			isSubSet: true,
		},
		"anyOf < everybody": {
			superSet: AccessConfig{Permission: AccessTypeEverybody},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"foobar"}},
			isSubSet: true,
		},
		"everybody <= everybody": {
			superSet: AccessConfig{Permission: AccessTypeEverybody},
			check:    AccessConfig{Permission: AccessTypeEverybody},
			isSubSet: true,
		},
		"unspecified !< everybody ": {
			superSet: AccessConfig{Permission: AccessTypeEverybody},
			check:    AccessConfig{Permission: AccessTypeUnspecified},
			isSubSet: false,
		},
		// unspecified
		"nobody !< unspecified": {
			superSet: AccessConfig{Permission: AccessTypeUnspecified},
			check:    AccessConfig{Permission: AccessTypeNobody},
			isSubSet: false,
		},
		"anyOf !< unspecified": {
			superSet: AccessConfig{Permission: AccessTypeUnspecified},
			check:    AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{"foobar"}},
			isSubSet: false,
		},
		"everybody !< unspecified": {
			superSet: AccessConfig{Permission: AccessTypeUnspecified},
			check:    AccessConfig{Permission: AccessTypeEverybody},
			isSubSet: false,
		},
		"unspecified !< unspecified ": {
			superSet: AccessConfig{Permission: AccessTypeUnspecified},
			check:    AccessConfig{Permission: AccessTypeUnspecified},
			isSubSet: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			subset := spec.check.IsSubset(spec.superSet)
			require.Equal(t, spec.isSubSet, subset)
		})
	}
}

func TestAccessTypeSubset(t *testing.T) {
	specs := map[string]struct {
		check    AccessType
		superSet AccessType
		isSubSet bool
	}{
		// nobody
		"nobody <= nobody": {
			superSet: AccessTypeNobody,
			check:    AccessTypeNobody,
			isSubSet: true,
		},
		"any !< nobody": {
			superSet: AccessTypeNobody,
			check:    AccessTypeAnyOfAddresses,
			isSubSet: false,
		},
		"everybody !< nobody": {
			superSet: AccessTypeNobody,
			check:    AccessTypeEverybody,
			isSubSet: false,
		},
		"unspecified !< nobody": {
			superSet: AccessTypeNobody,
			check:    AccessTypeUnspecified,
			isSubSet: false,
		},
		// any of
		"nobody < anyOf": {
			superSet: AccessTypeAnyOfAddresses,
			check:    AccessTypeNobody,
			isSubSet: true,
		},
		"anyOf <= anyOf": {
			superSet: AccessTypeAnyOfAddresses,
			check:    AccessTypeAnyOfAddresses,
			isSubSet: true,
		},
		"everybody !< anyOf": {
			superSet: AccessTypeAnyOfAddresses,
			check:    AccessTypeEverybody,
			isSubSet: false,
		},
		"unspecified !< anyOf": {
			superSet: AccessTypeAnyOfAddresses,
			check:    AccessTypeUnspecified,
			isSubSet: false,
		},
		// everybody
		"nobody < everybody": {
			superSet: AccessTypeEverybody,
			check:    AccessTypeNobody,
			isSubSet: true,
		},
		"anyOf < everybody": {
			superSet: AccessTypeEverybody,
			check:    AccessTypeAnyOfAddresses,
			isSubSet: true,
		},
		"everybody <= everybody": {
			superSet: AccessTypeEverybody,
			check:    AccessTypeEverybody,
			isSubSet: true,
		},
		"unspecified !< everybody": {
			superSet: AccessTypeEverybody,
			check:    AccessTypeUnspecified,
			isSubSet: false,
		},
		// unspecified
		"nobody !< unspecified": {
			superSet: AccessTypeUnspecified,
			check:    AccessTypeNobody,
			isSubSet: false,
		},
		"anyOf !< unspecified": {
			superSet: AccessTypeUnspecified,
			check:    AccessTypeAnyOfAddresses,
			isSubSet: false,
		},
		"everybody !< unspecified": {
			superSet: AccessTypeUnspecified,
			check:    AccessTypeEverybody,
			isSubSet: false,
		},
		"unspecified !< unspecified": {
			superSet: AccessTypeUnspecified,
			check:    AccessTypeUnspecified,
			isSubSet: false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			subset := spec.check.IsSubset(spec.superSet)
			require.Equal(t, spec.isSubSet, subset)
		})
	}
}

func TestContractCodeHistoryEntryValidation(t *testing.T) {
	specs := map[string]struct {
		src    ContractCodeHistoryEntry
		expErr bool
	}{
		"all good": {
			src: ContractCodeHistoryEntryFixture(),
		},
		"unknown operation": {
			src: ContractCodeHistoryEntryFixture(func(entry *ContractCodeHistoryEntry) {
				entry.Operation = 0
			}),
			expErr: true,
		},
		"empty code id": {
			src: ContractCodeHistoryEntryFixture(func(entry *ContractCodeHistoryEntry) {
				entry.CodeID = 0
			}),
			expErr: true,
		},
		"empty updated": {
			src: ContractCodeHistoryEntryFixture(func(entry *ContractCodeHistoryEntry) {
				entry.Updated = nil
			}),
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotErr := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
		})
	}
}
