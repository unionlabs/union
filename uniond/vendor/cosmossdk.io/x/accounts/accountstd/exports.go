// Package accountstd exports the types and functions that are used by developers to implement smart accounts.
package accountstd

import (
	"bytes"
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/core/transaction"
	"cosmossdk.io/x/accounts/internal/implementation"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

var (
	accountsModuleAddress = address.Module("accounts")
	ErrInvalidType        = errors.New("invalid type")
)

// Interface is the exported interface of an Account.
type Interface = implementation.Account

// ExecuteBuilder is the exported type of ExecuteBuilder.
type ExecuteBuilder = implementation.ExecuteBuilder

// QueryBuilder is the exported type of QueryBuilder.
type QueryBuilder = implementation.QueryBuilder

// InitBuilder is the exported type of InitBuilder.
type InitBuilder = implementation.InitBuilder

// AccountCreatorFunc is the exported type of AccountCreatorFunc.
type AccountCreatorFunc = implementation.AccountCreatorFunc

func DIAccount[A Interface](name string, constructor func(deps Dependencies) (A, error)) DepinjectAccount {
	return DepinjectAccount{MakeAccount: AddAccount(name, constructor)}
}

type DepinjectAccount struct {
	MakeAccount AccountCreatorFunc
}

func (DepinjectAccount) IsManyPerContainerType() {}

// Dependencies is the exported type of Dependencies.
type Dependencies = implementation.Dependencies

func RegisterExecuteHandler[
	Req any, ProtoReq implementation.ProtoMsgG[Req], Resp any, ProtoResp implementation.ProtoMsgG[Resp],
](router *ExecuteBuilder, handler func(ctx context.Context, req ProtoReq) (ProtoResp, error),
) {
	implementation.RegisterExecuteHandler(router, handler)
}

// RegisterQueryHandler registers a query handler for a smart account that uses protobuf.
func RegisterQueryHandler[
	Req any, ProtoReq implementation.ProtoMsgG[Req], Resp any, ProtoResp implementation.ProtoMsgG[Resp],
](router *QueryBuilder, handler func(ctx context.Context, req ProtoReq) (ProtoResp, error),
) {
	implementation.RegisterQueryHandler(router, handler)
}

// RegisterInitHandler registers an initialisation handler for a smart account that uses protobuf.
func RegisterInitHandler[
	Req any, ProtoReq implementation.ProtoMsgG[Req], Resp any, ProtoResp implementation.ProtoMsgG[Resp],
](router *InitBuilder, handler func(ctx context.Context, req ProtoReq) (ProtoResp, error),
) {
	implementation.RegisterInitHandler(router, handler)
}

// AddAccount is a helper function to add a smart account to the list of smart accounts.
func AddAccount[A Interface](name string, constructor func(deps Dependencies) (A, error)) AccountCreatorFunc {
	return func(deps implementation.Dependencies) (string, implementation.Account, error) {
		acc, err := constructor(deps)
		return name, acc, err
	}
}

// Whoami returns the address of the account being invoked.
func Whoami(ctx context.Context) []byte {
	return implementation.Whoami(ctx)
}

// Sender returns the sender of the execution request.
func Sender(ctx context.Context) []byte {
	return implementation.Sender(ctx)
}

// HasSender checks if the execution context was sent from the provided sender
func HasSender(ctx context.Context, wantSender []byte) bool {
	return bytes.Equal(Sender(ctx), wantSender)
}

// SenderIsSelf checks if the sender of the request is the account itself.
func SenderIsSelf(ctx context.Context) bool { return HasSender(ctx, Whoami(ctx)) }

// SenderIsAccountsModule returns true if the sender of the execution request is the accounts module.
func SenderIsAccountsModule(ctx context.Context) bool {
	return bytes.Equal(Sender(ctx), accountsModuleAddress)
}

// Funds returns if any funds were sent during the execute or init request. In queries this
// returns nil.
func Funds(ctx context.Context) sdk.Coins { return implementation.Funds(ctx) }

func ExecModule[MsgResp, Msg transaction.Msg](ctx context.Context, msg Msg) (resp MsgResp, err error) {
	untyped, err := implementation.ExecModule(ctx, msg)
	if err != nil {
		return resp, err
	}
	return assertOrErr[MsgResp](untyped)
}

// QueryModule can be used by an account to execute a module query.
func QueryModule[Resp, Req transaction.Msg](ctx context.Context, req Req) (resp Resp, err error) {
	untyped, err := implementation.QueryModule(ctx, req)
	if err != nil {
		return resp, err
	}
	return assertOrErr[Resp](untyped)
}

// UnpackAny unpacks a protobuf Any message generically.
func UnpackAny[Msg any, ProtoMsg implementation.ProtoMsgG[Msg]](any *implementation.Any) (*Msg, error) {
	return implementation.UnpackAny[Msg, ProtoMsg](any)
}

// PackAny packs a protobuf Any message generically.
func PackAny(msg transaction.Msg) (*implementation.Any, error) {
	return implementation.PackAny(msg)
}

// ExecModuleAnys can be used to execute a list of messages towards a module
// when those messages are packed in Any messages. The function returns a list
// of responses packed in Any messages.
func ExecModuleAnys(ctx context.Context, msgs []*implementation.Any) ([]*implementation.Any, error) {
	responses := make([]*implementation.Any, len(msgs))
	for i, msg := range msgs {
		concreteMessage, err := implementation.UnpackAnyRaw(msg)
		if err != nil {
			return nil, fmt.Errorf("error unpacking message %d: %w", i, err)
		}
		resp, err := implementation.ExecModule(ctx, concreteMessage)
		if err != nil {
			return nil, fmt.Errorf("error executing message %d: %w", i, err)
		}
		// pack again
		respAnyPB, err := implementation.PackAny(resp)
		if err != nil {
			return nil, fmt.Errorf("error packing response %d: %w", i, err)
		}
		responses[i] = respAnyPB
	}
	return responses, nil
}

// asserts the given any to the provided generic, returns ErrInvalidType if it can't.
func assertOrErr[T any](r any) (concrete T, err error) {
	concrete, ok := r.(T)
	if !ok {
		return concrete, ErrInvalidType
	}
	return concrete, nil
}
