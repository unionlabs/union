package group

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// MemberRequests defines a repeated slice of MemberRequest objects.
type MemberRequests struct {
	Members []MemberRequest
}

// ValidateBasic performs stateless validation on an array of members. On top
// of validating each member individually, it also makes sure there are no
// duplicate addresses.
func (ms MemberRequests) ValidateBasic() error {
	index := make(map[string]struct{}, len(ms.Members))
	for i := range ms.Members {
		member := ms.Members[i]
		if err := member.ValidateBasic(); err != nil {
			return err
		}
		addr := member.Address
		if _, exists := index[addr]; exists {
			return sdkerrors.Wrapf(errors.ErrDuplicate, "address: %s", addr)
		}
		index[addr] = struct{}{}
	}
	return nil
}

type accAddresses []sdk.AccAddress

// ValidateBasic verifies that there's no duplicate address.
// Individual account address validation has to be done separately.
func (a accAddresses) ValidateBasic() error {
	index := make(map[string]struct{}, len(a))
	for i := range a {
		accAddr := a[i]
		addr := string(accAddr)
		if _, exists := index[addr]; exists {
			return sdkerrors.Wrapf(errors.ErrDuplicate, "address: %s", accAddr.String())
		}
		index[addr] = struct{}{}
	}
	return nil
}
