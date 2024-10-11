package keeper

import (
	"context"
)

type Hooks struct {
	AfterCounterIncreased bool
}

func (h *Hooks) AfterIncreaseCount(ctx context.Context, n int64) error {
	h.AfterCounterIncreased = true
	return nil
}
