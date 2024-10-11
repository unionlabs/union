package testdata

import (
	"context"
	"fmt"
)

// This returns the head state.
// It does a full copy on head state for immutability.
func headState(ctx context.Context) {
	ctx, span := StartSpan(ctx, "blockChain.headState")
	fmt.Print(span)
	return
}

// StartSpan --
func StartSpan(ctx context.Context, name string) (context.Context, int) {
	return ctx, 42
}
