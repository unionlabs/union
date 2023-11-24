package types

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// UpgradeInfoFileName file to store upgrade information
const UpgradeInfoFilename = "upgrade-info.json"

// ValidateBasic does basic validation of a Plan
func (p Plan) ValidateBasic() error {
	if !p.Time.IsZero() {
		return sdkerrors.ErrInvalidRequest.Wrap("time-based upgrades have been deprecated in the SDK")
	}
	if p.UpgradedClientState != nil {
		return sdkerrors.ErrInvalidRequest.Wrap("upgrade logic for IBC has been moved to the IBC module")
	}
	if len(p.Name) == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "name cannot be empty")
	}
	if p.Height <= 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "height must be greater than 0")
	}

	return nil
}

// ShouldExecute returns true if the Plan is ready to execute given the current block height
func (p Plan) ShouldExecute(blockHeight int64) bool {
	return p.Height > 0 && p.Height <= blockHeight
}

// DueAt is a string representation of when this plan is due to be executed
func (p Plan) DueAt() string {
	return fmt.Sprintf("height: %d", p.Height)
}
