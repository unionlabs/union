package implementation

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/core/transaction"
)

var (
	errNoInitHandler    = errors.New("no init handler")
	errNoExecuteHandler = errors.New("account does not accept messages")
	errInvalidMessage   = errors.New("invalid message")
)

// NewInitBuilder creates a new InitBuilder instance.
func NewInitBuilder() *InitBuilder {
	return &InitBuilder{}
}

// InitBuilder defines a smart account's initialisation handler builder.
type InitBuilder struct {
	// handler is the handler function that will be called when the smart account is initialized.
	// Although the function here is defined to take an any, the smart account will work
	// with a typed version of it.
	handler func(ctx context.Context, initRequest transaction.Msg) (initResponse transaction.Msg, err error)

	// schema is the schema of the message that will be passed to the handler function.
	schema HandlerSchema
}

// makeHandler returns the handler function that will be called when the smart account is initialized.
// It returns an error if no handler was registered.
func (i *InitBuilder) makeHandler() (func(ctx context.Context, initRequest transaction.Msg) (initResponse transaction.Msg, err error), error) {
	if i.handler == nil {
		return nil, errNoInitHandler
	}
	return i.handler, nil
}

// NewExecuteBuilder creates a new ExecuteBuilder instance.
func NewExecuteBuilder() *ExecuteBuilder {
	return &ExecuteBuilder{
		handlers:       make(map[string]func(ctx context.Context, executeRequest transaction.Msg) (executeResponse transaction.Msg, err error)),
		handlersSchema: make(map[string]HandlerSchema),
	}
}

// ExecuteBuilder defines a smart account's execution router, it will be used to map an execution message
// to a handler function for a specific account.
type ExecuteBuilder struct {
	// handlers is a map of handler functions that will be called when the smart account is executed.
	handlers map[string]func(ctx context.Context, executeRequest transaction.Msg) (executeResponse transaction.Msg, err error)

	// handlersSchema is a map of schemas for the messages that will be passed to the handler functions
	// and the messages that will be returned by the handler functions.
	handlersSchema map[string]HandlerSchema

	// err is the error that occurred before building the handler function.
	err error
}

func (r *ExecuteBuilder) makeHandler() (func(ctx context.Context, executeRequest transaction.Msg) (executeResponse transaction.Msg, err error), error) {
	// if no handler is registered it's fine, it means the account will not be accepting execution or query messages.
	if len(r.handlers) == 0 {
		return func(ctx context.Context, _ transaction.Msg) (_ transaction.Msg, err error) {
			return nil, errNoExecuteHandler
		}, nil
	}

	if r.err != nil {
		return nil, r.err
	}

	// build the real execution handler
	return func(ctx context.Context, executeRequest transaction.Msg) (executeResponse transaction.Msg, err error) {
		messageName := MessageName(executeRequest)
		handler, ok := r.handlers[messageName]
		if !ok {
			return nil, fmt.Errorf("%w: no handler for message %s", errInvalidMessage, messageName)
		}
		return handler(ctx, executeRequest)
	}, nil
}

// NewQueryBuilder creates a new QueryBuilder instance.
func NewQueryBuilder() *QueryBuilder {
	return &QueryBuilder{
		er: NewExecuteBuilder(),
	}
}

// QueryBuilder defines a smart account's query router, it will be used to map a query message
// to a handler function for a specific account.
type QueryBuilder struct {
	// er is the ExecuteBuilder, since there's no difference between the execution and query handlers API.
	er *ExecuteBuilder
}

func (r *QueryBuilder) makeHandler() (func(ctx context.Context, queryRequest transaction.Msg) (queryResponse transaction.Msg, err error), error) {
	return r.er.makeHandler()
}

// IsRoutingError returns true if the error is a routing error,
// which typically occurs when a message cannot be matched to a handler.
func IsRoutingError(err error) bool {
	if err == nil {
		return false
	}
	return errors.Is(err, errInvalidMessage)
}
