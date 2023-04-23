package types

import (
	"fmt"
	"strings"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// NewControllerPortID creates and returns a new prefixed controller port identifier using the provided owner string
func NewControllerPortID(owner string) (string, error) {
	if strings.TrimSpace(owner) == "" {
		return "", sdkerrors.Wrap(ErrInvalidAccountAddress, "owner address cannot be empty")
	}

	return fmt.Sprint(ControllerPortPrefix, owner), nil
}
