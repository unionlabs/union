/*
NOTE: Usage of x/params to manage parameters is deprecated in favor of x/gov
controlled execution of MsgUpdateParams messages. These types remains solely
for migration purposes and will be removed in a future release.
[#3621](https://github.com/cosmos/ibc-go/issues/3621)
*/
package types

import (
	fmt "fmt"

	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

// KeyMaxExpectedTimePerBlock is store's key for MaxExpectedTimePerBlock parameter
var KeyMaxExpectedTimePerBlock = []byte("MaxExpectedTimePerBlock")

// ParamKeyTable type declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// ParamSetPairs implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyMaxExpectedTimePerBlock, &p.MaxExpectedTimePerBlock, validateParams),
	}
}

func validateParams(i interface{}) error {
	_, ok := i.(uint64)
	if !ok {
		return fmt.Errorf("invalid parameter. expected %T, got type: %T", uint64(1), i)
	}
	return nil
}
