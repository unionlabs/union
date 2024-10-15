package types

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// private type creates an interface key for Context that cannot be accessed by any other package
type contextKey int

const (
	// position counter of the TX in the block
	contextKeyTXCount contextKey = iota
	// smart query stack counter to abort query loops
	contextKeyQueryStackSize contextKey = iota
	// authorization policy for sub-messages
	contextKeySubMsgAuthzPolicy = iota
	// gas register
	contextKeyGasRegister = iota

	contextKeyCallDepth contextKey = iota
)

// WithTXCounter stores a transaction counter value in the context
func WithTXCounter(ctx sdk.Context, counter uint32) sdk.Context {
	return ctx.WithValue(contextKeyTXCount, counter)
}

// TXCounter returns the tx counter value and found bool from the context.
// The result will be (0, false) for external queries or simulations where no counter available.
func TXCounter(ctx context.Context) (uint32, bool) {
	val, ok := ctx.Value(contextKeyTXCount).(uint32)
	return val, ok
}

// WithQueryStackSize stores the stack position for smart queries in the context returned
func WithQueryStackSize(ctx sdk.Context, counter uint32) sdk.Context {
	return ctx.WithValue(contextKeyQueryStackSize, counter)
}

// QueryStackSize reads the stack position for smart queries from the context
func QueryStackSize(ctx context.Context) (uint32, bool) {
	val, ok := ctx.Value(contextKeyQueryStackSize).(uint32)
	return val, ok
}

func WithCallDepth(ctx sdk.Context, counter uint32) sdk.Context {
	return ctx.WithValue(contextKeyCallDepth, counter)
}

func CallDepth(ctx context.Context) (uint32, bool) {
	val, ok := ctx.Value(contextKeyCallDepth).(uint32)
	return val, ok
}

// WithSubMsgAuthzPolicy stores the authorization policy for submessages into the context returned
func WithSubMsgAuthzPolicy(ctx sdk.Context, policy AuthorizationPolicy) sdk.Context {
	if policy == nil {
		panic("policy must not be nil")
	}
	return ctx.WithValue(contextKeySubMsgAuthzPolicy, policy)
}

// SubMsgAuthzPolicy reads the authorization policy for submessages from the context
func SubMsgAuthzPolicy(ctx context.Context) (AuthorizationPolicy, bool) {
	val, ok := ctx.Value(contextKeySubMsgAuthzPolicy).(AuthorizationPolicy)
	return val, ok
}

// WithGasRegister stores the gas register into the context returned
func WithGasRegister(ctx sdk.Context, gr GasRegister) sdk.Context {
	if gr == nil {
		panic("gas register must not be nil")
	}
	return ctx.WithValue(contextKeyGasRegister, gr)
}

// GasRegisterFromContext reads the gas register from the context
func GasRegisterFromContext(ctx context.Context) (GasRegister, bool) {
	val, ok := ctx.Value(contextKeyGasRegister).(GasRegister)
	return val, ok
}
