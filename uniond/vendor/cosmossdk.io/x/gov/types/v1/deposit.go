package v1

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewDeposit creates a new Deposit instance
func NewDeposit(proposalID uint64, depositor string, amount sdk.Coins) Deposit {
	return Deposit{proposalID, depositor, amount}
}

// Deposits is a collection of Deposit objects
type Deposits []*Deposit

// Equal returns true if two slices (order-dependant) of deposits are equal.
func (d Deposits) Equal(other Deposits) bool {
	if len(d) != len(other) {
		return false
	}

	for i, deposit := range d {
		if deposit.String() != other[i].String() {
			return false
		}
	}

	return true
}

// String implements stringer interface
func (d Deposits) String() string {
	if len(d) == 0 {
		return "[]"
	}
	out := fmt.Sprintf("Deposits for Proposal %d:", d[0].ProposalId)
	for _, dep := range d {
		out += fmt.Sprintf("\n  %s: %s", dep.Depositor, dep.Amount)
	}
	return out
}
