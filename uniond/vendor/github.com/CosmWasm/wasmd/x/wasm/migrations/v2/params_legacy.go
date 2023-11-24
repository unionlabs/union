/*
NOTE: Usage of x/params to manage parameters is deprecated in favor of x/gov
controlled execution of MsgUpdateParams messages. These types remains solely
for migration purposes and will be removed in a future release.
*/
package v2

import (
	"encoding/json"
	"fmt"

	"github.com/cosmos/gogoproto/jsonpb"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var (
	ParamStoreKeyUploadAccess      = []byte("uploadAccess")
	ParamStoreKeyInstantiateAccess = []byte("instantiateAccess")
)

// Deprecated: Type declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// ParamSetPairs returns the parameter set pairs.
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(ParamStoreKeyUploadAccess, &p.CodeUploadAccess, validateAccessConfig),
		paramtypes.NewParamSetPair(ParamStoreKeyInstantiateAccess, &p.InstantiateDefaultPermission, validateAccessType),
	}
}

func validateAccessConfig(i interface{}) error {
	v, ok := i.(AccessConfig)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	return v.ValidateBasic()
}

var AllAccessTypes = []AccessType{
	AccessTypeNobody,
	AccessTypeOnlyAddress,
	AccessTypeAnyOfAddresses,
	AccessTypeEverybody,
}

func validateAccessType(i interface{}) error {
	a, ok := i.(AccessType)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	if a == AccessTypeUnspecified {
		return errorsmod.Wrap(types.ErrEmpty, "type")
	}
	for _, v := range AllAccessTypes {
		if v == a {
			return nil
		}
	}
	return errorsmod.Wrapf(types.ErrInvalid, "unknown type: %q", a)
}

func (a AccessConfig) ValidateBasic() error {
	switch a.Permission {
	case AccessTypeUnspecified:
		return errorsmod.Wrap(types.ErrEmpty, "type")
	case AccessTypeNobody, AccessTypeEverybody:
		return nil
	case AccessTypeOnlyAddress:
		_, err := sdk.AccAddressFromBech32(a.Address)
		return errorsmod.Wrap(err, "only address")
	case AccessTypeAnyOfAddresses:
		return errorsmod.Wrap(assertValidAddresses(a.Addresses), "addresses")
	}
	return errorsmod.Wrapf(types.ErrInvalid, "unknown type: %q", a.Permission)
}

func assertValidAddresses(addrs []string) error {
	if len(addrs) == 0 {
		return types.ErrEmpty
	}
	idx := make(map[string]struct{}, len(addrs))
	for _, a := range addrs {
		if _, err := sdk.AccAddressFromBech32(a); err != nil {
			return errorsmod.Wrapf(err, "address: %s", a)
		}
		if _, exists := idx[a]; exists {
			return types.ErrDuplicate.Wrapf("address: %s", a)
		}
		idx[a] = struct{}{}
	}
	return nil
}

func (a AccessType) String() string {
	switch a {
	case AccessTypeNobody:
		return "Nobody"
	case AccessTypeOnlyAddress:
		return "OnlyAddress"
	case AccessTypeEverybody:
		return "Everybody"
	case AccessTypeAnyOfAddresses:
		return "AnyOfAddresses"
	}
	return "Unspecified"
}

func (a *AccessType) UnmarshalText(text []byte) error {
	for _, v := range AllAccessTypes {
		if v.String() == string(text) {
			*a = v
			return nil
		}
	}
	*a = AccessTypeUnspecified
	return nil
}

func (a AccessType) MarshalText() ([]byte, error) {
	return []byte(a.String()), nil
}

func (a *AccessType) MarshalJSONPB(_ *jsonpb.Marshaler) ([]byte, error) {
	return json.Marshal(a)
}

func (a *AccessType) UnmarshalJSONPB(_ *jsonpb.Unmarshaler, data []byte) error {
	return json.Unmarshal(data, a)
}

func (a AccessConfig) Equals(o AccessConfig) bool {
	return a.Permission == o.Permission && a.Address == o.Address
}
