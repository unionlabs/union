package v2

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
)

func TestAccessTypeMarshalJson(t *testing.T) {
	specs := map[string]struct {
		src AccessType
		exp string
	}{
		"Unspecified":              {src: AccessTypeUnspecified, exp: `"Unspecified"`},
		"Nobody":                   {src: AccessTypeNobody, exp: `"Nobody"`},
		"OnlyAddress":              {src: AccessTypeOnlyAddress, exp: `"OnlyAddress"`},
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
		"OnlyAddress":    {src: `"OnlyAddress"`, exp: AccessTypeOnlyAddress},
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
			exp: Params{
				CodeUploadAccess:             AccessConfig{Permission: AccessTypeEverybody},
				InstantiateDefaultPermission: AccessTypeEverybody,
			},
		},
		"legacy type": {
			src: `{"code_upload_access": {"permission": "OnlyAddress", "address": "wasm1lq3q55r9sqwqyrfmlp6xy8ufhayt96lmcttthz", "addresses": [] },
					"instantiate_default_permission": "Nobody"}`,
			exp: Params{
				CodeUploadAccess: AccessConfig{
					Permission: AccessTypeOnlyAddress, Address: "wasm1lq3q55r9sqwqyrfmlp6xy8ufhayt96lmcttthz", Addresses: nil,
				},
				InstantiateDefaultPermission: AccessTypeNobody,
			},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			var val Params
			marshaler := codec.NewLegacyAmino()
			err := marshaler.UnmarshalJSON([]byte(spec.src), &val)
			require.NoError(t, err)
			assert.Equal(t, spec.exp, val)
		})
	}
}
