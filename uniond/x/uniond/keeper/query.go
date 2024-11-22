package keeper

import (
	"github.com/unionlabs/union/uniond/x/uniond/types"
)

var _ types.QueryServer = Keeper{}
