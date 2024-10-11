package appmodulev2

import (
	"context"
	"fmt"

	transaction "cosmossdk.io/core/transaction"
)

type (
	// PreMsgHandler is a handler that is executed before Handler. If it errors the execution reverts.
	PreMsgHandler = func(ctx context.Context, msg transaction.Msg) error
	// HandlerFunc handles the state transition of the provided message.
	HandlerFunc = func(ctx context.Context, msg transaction.Msg) (msgResp transaction.Msg, err error)
	// PostMsgHandler runs after Handler, only if Handler does not error. If PostMsgHandler errors
	// then the execution is reverted.
	PostMsgHandler = func(ctx context.Context, msg, msgResp transaction.Msg) error
)

// PreMsgRouter is a router that allows you to register PreMsgHandlers for specific message types.
type PreMsgRouter interface {
	// RegisterPreMsgHandler will register a specific message handler hooking into the message with
	// the provided name.
	RegisterPreMsgHandler(msgName string, handler PreMsgHandler)
	// RegisterGlobalPreMsgHandler will register a global message handler hooking into any message
	// being executed.
	RegisterGlobalPreMsgHandler(handler PreMsgHandler)
}

// HasPreMsgHandlers is an interface that modules must implement if they want to register PreMsgHandlers.
type HasPreMsgHandlers interface {
	RegisterPreMsgHandlers(router PreMsgRouter)
}

// RegisterMsgPreHandler is a helper function that modules can use to not lose type safety when registering PreMsgHandler to the
// PreMsgRouter. Example usage:
// ```go
//
//	func (h Handlers) BeforeSend(ctx context.Context, req *types.MsgSend) error {
//	      ... before send logic ...
//	}
//
//	func (m Module) RegisterPreMsgHandlers(router appmodule.PreMsgRouter) {
//		handlers := keeper.NewHandlers(m.keeper)
//	    appmodule.RegisterMsgPreHandler(router, gogoproto.MessageName(types.MsgSend{}), handlers.BeforeSend)
//	}
//
// ```
func RegisterMsgPreHandler[Req transaction.Msg](
	router PreMsgRouter,
	msgName string,
	handler func(ctx context.Context, msg Req) error,
) {
	untypedHandler := func(ctx context.Context, m transaction.Msg) error {
		typed, ok := m.(Req)
		if !ok {
			return fmt.Errorf("unexpected type %T, wanted: %T", m, *new(Req))
		}
		return handler(ctx, typed)
	}

	router.RegisterPreMsgHandler(msgName, untypedHandler)
}

// PostMsgRouter is a router that allows you to register PostMsgHandlers for specific message types.
type PostMsgRouter interface {
	// RegisterPostMsgHandler will register a specific message handler hooking after the execution of message with
	// the provided name.
	RegisterPostMsgHandler(msgName string, handler PostMsgHandler)
	// RegisterGlobalPostMsgHandler will register a global message handler hooking after the execution of any message.
	RegisterGlobalPostMsgHandler(handler PostMsgHandler)
}

// HasPostMsgHandlers is an interface that modules must implement if they want to register PostMsgHandlers.
type HasPostMsgHandlers interface {
	RegisterPostMsgHandlers(router PostMsgRouter)
}

// RegisterPostMsgHandler is a helper function that modules can use to not lose type safety when registering handlers to the
// PostMsgRouter. Example usage:
// ```go
//
//	func (h Handlers) AfterSend(ctx context.Context, req *types.MsgSend, resp *types.MsgSendResponse) error {
//	      ... query logic ...
//	}
//
//	func (m Module) RegisterPostMsgHandlers(router appmodule.PostMsgRouter) {
//		handlers := keeper.NewHandlers(m.keeper)
//	    appmodule.RegisterPostMsgHandler(router, gogoproto.MessageName(types.MsgSend{}), handlers.AfterSend)
//	}
//
// ```
func RegisterPostMsgHandler[Req, Resp transaction.Msg](
	router PostMsgRouter,
	msgName string,
	handler func(ctx context.Context, msg Req, msgResp Resp) error,
) {
	untypedHandler := func(ctx context.Context, m, mResp transaction.Msg) error {
		typed, ok := m.(Req)
		if !ok {
			return fmt.Errorf("unexpected type %T, wanted: %T", m, *new(Req))
		}
		typedResp, ok := mResp.(Resp)
		if !ok {
			return fmt.Errorf("unexpected type %T, wanted: %T", m, *new(Resp))
		}
		return handler(ctx, typed, typedResp)
	}

	router.RegisterPostMsgHandler(msgName, untypedHandler)
}

// Handler defines a handler descriptor.
type Handler struct {
	// Func defines the actual handler, the function that runs a request and returns a response.
	// Can be query handler or msg handler.
	Func HandlerFunc
	// MakeMsg instantiates the type of the request, can be used in decoding contexts.
	MakeMsg func() transaction.Msg
	// MakeMsgResp instantiates a new response, can be used in decoding contexts.
	MakeMsgResp func() transaction.Msg
}

// MsgRouter is a router that allows you to register Handlers for specific message types.
type MsgRouter = interface {
	RegisterHandler(handler Handler)
}

// HasMsgHandlers is an interface that modules must implement if they want to register Handlers.
type HasMsgHandlers interface {
	RegisterMsgHandlers(router MsgRouter)
}

// QueryRouter is a router that allows you to register QueryHandlers for specific query types.
type QueryRouter = MsgRouter

// HasQueryHandlers is an interface that modules must implement if they want to register QueryHandlers.
type HasQueryHandlers interface {
	RegisterQueryHandlers(router QueryRouter)
}

// RegisterMsgHandler is a helper function that modules can use to not lose type safety when registering handlers to the MsgRouter and Query Router.
// Example usage:
// ```go
//
//	func (h Handlers) Mint(ctx context.Context, req *types.MsgMint) (*types.MsgMintResponse, error) {
//	      ... msg logic ...
//	}
//
//	func (h Handlers) QueryBalance(ctx context.Context, req *types.QueryBalanceRequest) (*types.QueryBalanceResponse, error) {
//	      ... query logic ...
//	}
//
//	func (m Module) RegisterMsgHandlers(router appmodule.MsgRouter) {
//		handlers := keeper.NewHandlers(m.keeper)
//	    err := appmodule.RegisterHandler(router, handlers.MsgMint)
//	}
//
//	func (m Module) RegisterQueryHandlers(router appmodule.QueryRouter) {
//		handlers := keeper.NewHandlers(m.keeper)
//	    err := appmodule.RegisterHandler(router, handlers.QueryBalance)
//	}
//
// ```
func RegisterMsgHandler[Req, Resp any, PReq transaction.GenericMsg[Req], PResp transaction.GenericMsg[Resp]](
	router MsgRouter,
	handler func(ctx context.Context, msg PReq) (msgResp PResp, err error),
) {
	untypedHandler := func(ctx context.Context, m transaction.Msg) (transaction.Msg, error) {
		typed, ok := m.(PReq)
		if !ok {
			return nil, fmt.Errorf("unexpected type %T, wanted: %T", m, *new(Req))
		}
		return handler(ctx, typed)
	}

	router.RegisterHandler(Handler{
		Func: untypedHandler,
		MakeMsg: func() transaction.Msg {
			return PReq(new(Req))
		},
		MakeMsgResp: func() transaction.Msg {
			return PResp(new(Resp))
		},
	})
}
