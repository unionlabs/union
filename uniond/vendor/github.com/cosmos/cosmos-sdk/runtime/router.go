package runtime

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"strings"

	gogoproto "github.com/cosmos/gogoproto/proto"
	protov2 "google.golang.org/protobuf/proto"

	"cosmossdk.io/core/router"

	"github.com/cosmos/cosmos-sdk/baseapp"
)

// NewMsgRouterService return new implementation of router.Service.
func NewMsgRouterService(msgRouter baseapp.MessageRouter) router.Service {
	return &msgRouterService{
		router: msgRouter,
	}
}

var _ router.Service = (*msgRouterService)(nil)

type msgRouterService struct {
	// TODO: eventually authenticate modules to use the message router
	router baseapp.MessageRouter
}

// CanInvoke returns an error if the given message cannot be invoked.
func (m *msgRouterService) CanInvoke(ctx context.Context, typeURL string) error {
	if typeURL == "" {
		return errors.New("missing type url")
	}

	typeURL = strings.TrimPrefix(typeURL, "/")

	handler := m.router.HybridHandlerByMsgName(typeURL)
	if handler == nil {
		return fmt.Errorf("unknown message: %s", typeURL)
	}

	return nil
}

// Invoke execute a message and returns a response.
func (m *msgRouterService) Invoke(ctx context.Context, msg gogoproto.Message) (gogoproto.Message, error) {
	messageName := msgTypeURL(msg)
	respName := m.router.ResponseNameByMsgName(messageName)
	if respName == "" {
		return nil, fmt.Errorf("could not find response type for message %s (%T)", messageName, msg)
	}

	// get response type
	typ := gogoproto.MessageType(respName)
	if typ == nil {
		return nil, fmt.Errorf("no message type found for %s", respName)
	}
	msgResp, ok := reflect.New(typ.Elem()).Interface().(gogoproto.Message)
	if !ok {
		return nil, fmt.Errorf("could not create response message %s", respName)
	}

	handler := m.router.HybridHandlerByMsgName(messageName)
	if handler == nil {
		return nil, fmt.Errorf("unknown message: %s", messageName)
	}

	if err := handler(ctx, msg, msgResp); err != nil {
		return nil, err
	}

	return msgResp, nil
}

// NewQueryRouterService return new implementation of router.Service.
func NewQueryRouterService(queryRouter baseapp.QueryRouter) router.Service {
	return &queryRouterService{
		router: queryRouter,
	}
}

var _ router.Service = (*queryRouterService)(nil)

type queryRouterService struct {
	router baseapp.QueryRouter
}

// CanInvoke returns an error if the given request cannot be invoked.
func (m *queryRouterService) CanInvoke(ctx context.Context, typeURL string) error {
	if typeURL == "" {
		return errors.New("missing type url")
	}

	typeURL = strings.TrimPrefix(typeURL, "/")

	handlers := m.router.HybridHandlerByRequestName(typeURL)
	if len(handlers) == 0 {
		return fmt.Errorf("unknown request: %s", typeURL)
	} else if len(handlers) > 1 {
		return fmt.Errorf("ambiguous request, query have multiple handlers: %s", typeURL)
	}

	return nil
}

// Invoke execute a message and returns a response.
func (m *queryRouterService) Invoke(ctx context.Context, req gogoproto.Message) (gogoproto.Message, error) {
	reqName := msgTypeURL(req)
	respName := m.router.ResponseNameByRequestName(reqName)
	if respName == "" {
		return nil, fmt.Errorf("unknown request: could not find response type for request %s (%T)", reqName, req)
	}

	// get response type
	typ := gogoproto.MessageType(respName)
	if typ == nil {
		return nil, fmt.Errorf("no message type found for %s", respName)
	}
	reqResp, ok := reflect.New(typ.Elem()).Interface().(gogoproto.Message)
	if !ok {
		return nil, fmt.Errorf("could not create response request %s", respName)
	}

	handlers := m.router.HybridHandlerByRequestName(reqName)
	if len(handlers) == 0 {
		return nil, fmt.Errorf("unknown request: %s", reqName)
	} else if len(handlers) > 1 {
		return nil, fmt.Errorf("ambiguous request, query have multiple handlers: %s", reqName)
	}

	if err := handlers[0](ctx, req, reqResp); err != nil {
		return nil, err
	}

	return reqResp, nil
}

// msgTypeURL returns the TypeURL of a proto message.
func msgTypeURL(msg gogoproto.Message) string {
	if m, ok := msg.(protov2.Message); ok {
		return string(m.ProtoReflect().Descriptor().FullName())
	}

	return gogoproto.MessageName(msg)
}
