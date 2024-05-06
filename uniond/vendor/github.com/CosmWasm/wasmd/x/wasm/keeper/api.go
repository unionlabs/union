package keeper

import (
	"errors"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

const (
	// DefaultGasCostHumanAddress is how much SDK gas we charge to convert to a human address format
	DefaultGasCostHumanAddress = 5
	// DefaultGasCostCanonicalAddress is how much SDK gas we charge to convert to a canonical address format
	DefaultGasCostCanonicalAddress = 4
	// DefaultGasCostValidateAddress is how much SDK gas we charge to validate an address
	DefaultGasCostValidateAddress = DefaultGasCostHumanAddress + DefaultGasCostCanonicalAddress

	// DefaultDeserializationCostPerByte The formula should be `len(data) * deserializationCostPerByte`
	DefaultDeserializationCostPerByte = 1
)

var (
	costHumanize            = DefaultGasCostHumanAddress * types.DefaultGasMultiplier
	costCanonical           = DefaultGasCostCanonicalAddress * types.DefaultGasMultiplier
	costValidate            = DefaultGasCostValidateAddress * types.DefaultGasMultiplier
	costJSONDeserialization = wasmvmtypes.UFraction{
		Numerator:   DefaultDeserializationCostPerByte * types.DefaultGasMultiplier,
		Denominator: 1,
	}
)

func humanizeAddress(canon []byte) (string, uint64, error) {
	if err := sdk.VerifyAddressFormat(canon); err != nil {
		return "", costHumanize, err
	}
	return sdk.AccAddress(canon).String(), costHumanize, nil
}

func canonicalizeAddress(human string) ([]byte, uint64, error) {
	bz, err := sdk.AccAddressFromBech32(human)
	return bz, costCanonical, err
}

func validateAddress(human string) (uint64, error) {
	canonicalized, err := sdk.AccAddressFromBech32(human)
	if err != nil {
		return costValidate, err
	}
	// AccAddressFromBech32 already calls VerifyAddressFormat, so we can just humanize and compare
	if canonicalized.String() != human {
		return costValidate, errors.New("address not normalized")
	}
	return costValidate, nil
}

var cosmwasmAPI = wasmvm.GoAPI{
	HumanizeAddress:     humanizeAddress,
	CanonicalizeAddress: canonicalizeAddress,
	ValidateAddress:     validateAddress,
}
