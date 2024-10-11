package types

import (
	"encoding/json"

	"github.com/cosmos/gogoproto/jsonpb"
	"github.com/pkg/errors"
	"gopkg.in/yaml.v2"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var AllAccessTypes = []AccessType{
	AccessTypeNobody,
	AccessTypeAnyOfAddresses,
	AccessTypeEverybody,
}

func (a AccessType) With(addrs ...sdk.AccAddress) AccessConfig {
	switch a {
	case AccessTypeNobody:
		return AllowNobody
	case AccessTypeEverybody:
		return AllowEverybody
	case AccessTypeAnyOfAddresses:
		bech32Addrs := make([]string, len(addrs))
		for i, v := range addrs {
			bech32Addrs[i] = v.String()
		}
		if err := validateBech32Addresses(bech32Addrs); err != nil {
			panic(errorsmod.Wrap(err, "addresses"))
		}
		return AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: bech32Addrs}
	}
	panic("unsupported access type")
}

func (a AccessType) String() string {
	switch a {
	case AccessTypeNobody:
		return "Nobody"
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
	return a.Permission == o.Permission
}

var (
	DefaultUploadAccess = AllowEverybody
	AllowEverybody      = AccessConfig{Permission: AccessTypeEverybody}
	AllowNobody         = AccessConfig{Permission: AccessTypeNobody}
)

// DefaultParams returns default wasm parameters
func DefaultParams() Params {
	return Params{
		CodeUploadAccess:             AllowEverybody,
		InstantiateDefaultPermission: AccessTypeEverybody,
	}
}

func (p Params) String() string {
	out, err := yaml.Marshal(p)
	if err != nil {
		panic(err)
	}
	return string(out)
}

// ValidateBasic performs basic validation on wasm parameters
func (p Params) ValidateBasic() error {
	if err := validateAccessType(p.InstantiateDefaultPermission); err != nil {
		return errors.Wrap(err, "instantiate default permission")
	}
	if err := p.CodeUploadAccess.ValidateBasic(); err != nil {
		return errors.Wrap(err, "upload access")
	}
	return nil
}

func validateAccessType(a AccessType) error {
	if a == AccessTypeUnspecified {
		return errorsmod.Wrap(ErrEmpty, "type")
	}
	for _, v := range AllAccessTypes {
		if v == a {
			return nil
		}
	}
	return errorsmod.Wrapf(ErrInvalid, "unknown type: %q", a)
}

// ValidateBasic performs basic validation
func (a AccessConfig) ValidateBasic() error {
	switch a.Permission {
	case AccessTypeUnspecified:
		return errorsmod.Wrap(ErrEmpty, "type")
	case AccessTypeNobody, AccessTypeEverybody:
		return nil
	case AccessTypeAnyOfAddresses:
		return errorsmod.Wrap(validateBech32Addresses(a.Addresses), "addresses")
	}
	return errorsmod.Wrapf(ErrInvalid, "unknown type: %q", a.Permission)
}

// Allowed returns if permission includes the actor.
// Actor address must be valid and not nil
func (a AccessConfig) Allowed(actor sdk.AccAddress) bool {
	switch a.Permission {
	case AccessTypeNobody:
		return false
	case AccessTypeEverybody:
		return true
	case AccessTypeAnyOfAddresses:
		for _, v := range a.Addresses {
			if v == actor.String() {
				return true
			}
		}
		return false
	default:
		panic("unknown type")
	}
}
