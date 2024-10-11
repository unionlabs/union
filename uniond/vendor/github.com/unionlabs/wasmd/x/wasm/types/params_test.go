package types

import (
	"bytes"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestValidateParams(t *testing.T) {
	var (
		anyAddress     sdk.AccAddress = make([]byte, ContractAddrLen)
		otherAddress   sdk.AccAddress = bytes.Repeat([]byte{1}, ContractAddrLen)
		invalidAddress                = "invalid address"
	)

	specs := map[string]struct {
		src    Params
		expErr bool
	}{
		"all good with defaults": {
			src: DefaultParams(),
		},
		"all good with nobody": {
			src: Params{
				CodeUploadAccess:             AllowNobody,
				InstantiateDefaultPermission: AccessTypeNobody,
			},
		},
		"all good with everybody": {
			src: Params{
				CodeUploadAccess:             AllowEverybody,
				InstantiateDefaultPermission: AccessTypeEverybody,
			},
		},
		"all good with anyOf address": {
			src: Params{
				CodeUploadAccess:             AccessTypeAnyOfAddresses.With(anyAddress),
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
		},
		"all good with anyOf addresses": {
			src: Params{
				CodeUploadAccess:             AccessTypeAnyOfAddresses.With(anyAddress, otherAddress),
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
		},
		"reject empty type in instantiate permission": {
			src: Params{
				CodeUploadAccess: AllowNobody,
			},
			expErr: true,
		},
		"reject unknown type in instantiate": {
			src: Params{
				CodeUploadAccess:             AllowNobody,
				InstantiateDefaultPermission: 1111,
			},
			expErr: true,
		},
		"reject empty CodeUploadAccess": {
			src: Params{
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
			expErr: true,
		},
		"reject undefined permission in CodeUploadAccess": {
			src: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeUnspecified},
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
			expErr: true,
		},
		"reject empty addresses in any of addresses": {
			src: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{}},
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
			expErr: true,
		},
		"reject addresses not set in any of addresses": {
			src: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeAnyOfAddresses},
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
			expErr: true,
		},
		"reject invalid address in any of addresses": {
			src: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{invalidAddress}},
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
			expErr: true,
		},
		"reject duplicate address in any of addresses": {
			src: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{anyAddress.String(), anyAddress.String()}},
				InstantiateDefaultPermission: AccessTypeAnyOfAddresses,
			},
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

func TestAccessTypeMarshalJson(t *testing.T) {
	specs := map[string]struct {
		src AccessType
		exp string
	}{
		"Unspecified":              {src: AccessTypeUnspecified, exp: `"Unspecified"`},
		"Nobody":                   {src: AccessTypeNobody, exp: `"Nobody"`},
		"AccessTypeAnyOfAddresses": {src: AccessTypeAnyOfAddresses, exp: `"AnyOfAddresses"`},
		"Everybody":                {src: AccessTypeEverybody, exp: `"Everybody"`},
		"unknown":                  {src: 999, exp: `"Unspecified"`},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			got, err := json.Marshal(spec.src)
			require.NoError(t, err)
			assert.Equal(t, []byte(spec.exp), got)
		})
	}
}

func TestAccessTypeUnmarshalJson(t *testing.T) {
	specs := map[string]struct {
		src string
		exp AccessType
	}{
		"Unspecified":    {src: `"Unspecified"`, exp: AccessTypeUnspecified},
		"Nobody":         {src: `"Nobody"`, exp: AccessTypeNobody},
		"AnyOfAddresses": {src: `"AnyOfAddresses"`, exp: AccessTypeAnyOfAddresses},
		"Everybody":      {src: `"Everybody"`, exp: AccessTypeEverybody},
		"unknown":        {src: `""`, exp: AccessTypeUnspecified},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			var got AccessType
			err := json.Unmarshal([]byte(spec.src), &got)
			require.NoError(t, err)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestParamsUnmarshalJson(t *testing.T) {
	specs := map[string]struct {
		src string
		exp Params
	}{
		"defaults": {
			src: `{"code_upload_access": {"permission": "Everybody"},
				"instantiate_default_permission": "Everybody"}`,
			exp: DefaultParams(),
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			var val Params
			interfaceRegistry := codectypes.NewInterfaceRegistry()
			marshaler := codec.NewProtoCodec(interfaceRegistry)

			err := marshaler.UnmarshalJSON([]byte(spec.src), &val)
			require.NoError(t, err)
			assert.Equal(t, spec.exp, val)
		})
	}
}

func TestAccessTypeWith(t *testing.T) {
	myAddress := sdk.AccAddress(randBytes(SDKAddrLen))
	myOtherAddress := sdk.AccAddress(randBytes(SDKAddrLen))
	specs := map[string]struct {
		src      AccessType
		addrs    []sdk.AccAddress
		exp      AccessConfig
		expPanic bool
	}{
		"nobody": {
			src: AccessTypeNobody,
			exp: AccessConfig{Permission: AccessTypeNobody},
		},
		"nobody with address": {
			src:   AccessTypeNobody,
			addrs: []sdk.AccAddress{myAddress},
			exp:   AccessConfig{Permission: AccessTypeNobody},
		},
		"everybody": {
			src: AccessTypeEverybody,
			exp: AccessConfig{Permission: AccessTypeEverybody},
		},
		"everybody with address": {
			src:   AccessTypeEverybody,
			addrs: []sdk.AccAddress{myAddress},
			exp:   AccessConfig{Permission: AccessTypeEverybody},
		},
		"any of address without address": {
			src:      AccessTypeAnyOfAddresses,
			expPanic: true,
		},
		"any of address with single address": {
			src:   AccessTypeAnyOfAddresses,
			addrs: []sdk.AccAddress{myAddress},
			exp:   AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{myAddress.String()}},
		},
		"any of address with multiple addresses": {
			src:   AccessTypeAnyOfAddresses,
			addrs: []sdk.AccAddress{myAddress, myOtherAddress},
			exp:   AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{myAddress.String(), myOtherAddress.String()}},
		},
		"any of address with duplicate addresses": {
			src:      AccessTypeAnyOfAddresses,
			addrs:    []sdk.AccAddress{myAddress, myAddress},
			expPanic: true,
		},
		"any of address with invalid address": {
			src:      AccessTypeAnyOfAddresses,
			addrs:    []sdk.AccAddress{nil},
			expPanic: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if !spec.expPanic {
				got := spec.src.With(spec.addrs...)
				assert.Equal(t, spec.exp, got)
				return
			}
			assert.Panics(t, func() {
				spec.src.With(spec.addrs...)
			})
		})
	}
}
