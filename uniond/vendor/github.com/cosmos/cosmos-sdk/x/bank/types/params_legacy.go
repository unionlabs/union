package types

import (
	fmt "fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/bank/exported"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

var (
	// KeySendEnabled is store's key for SendEnabled Params
	// Deprecated: Use the SendEnabled functionality in the keeper.
	KeySendEnabled = []byte("SendEnabled")
	// KeyDefaultSendEnabled is store's key for the DefaultSendEnabled option
	KeyDefaultSendEnabled = []byte("DefaultSendEnabled")
)

// Deprecated: ParamKeyTable for bank module.
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// Deprecated: ParamSetPairs implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeySendEnabled, &p.SendEnabled, validateSendEnabledParams),
		paramtypes.NewParamSetPair(KeyDefaultSendEnabled, &p.DefaultSendEnabled, validateIsBool),
	}
}

// SendEnabledParams is a collection of parameters indicating if a coin denom is enabled for sending
type SendEnabledParams []*SendEnabled

// GetSendEnabledParams retrieves the send enabled parameters from the provided context and legacy subspace.
func GetSendEnabledParams(ctx sdk.Context, legacySubspace exported.Subspace) []*SendEnabled {
	var sendEnabled []*SendEnabled
	legacySubspace.Get(ctx, KeySendEnabled, &sendEnabled)
	return sendEnabled
}

func validateSendEnabledParams(i interface{}) error {
	params, ok := i.([]*SendEnabled)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	// ensure each denom is only registered one time.
	registered := make(map[string]bool)
	for _, p := range params {
		if _, exists := registered[p.Denom]; exists {
			return fmt.Errorf("duplicate send enabled parameter found: '%s'", p.Denom)
		}
		if err := validateSendEnabled(*p); err != nil {
			return err
		}
		registered[p.Denom] = true
	}
	return nil
}

func validateSendEnabled(i interface{}) error {
	param, ok := i.(SendEnabled)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	return sdk.ValidateDenom(param.Denom)
}
