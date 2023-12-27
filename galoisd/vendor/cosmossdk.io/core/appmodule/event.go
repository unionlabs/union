package appmodule

import (
	"context"

	"google.golang.org/protobuf/runtime/protoiface"
)

// HasEventListeners is the extension interface that modules should implement to register
// event listeners.
type HasEventListeners interface {
	AppModule

	// RegisterEventListeners registers the module's events listeners.
	RegisterEventListeners(registrar *EventListenerRegistrar)
}

// EventListenerRegistrar allows registering event listeners.
type EventListenerRegistrar struct {
	listeners []any
}

// GetListeners gets the event listeners that have been registered
func (e *EventListenerRegistrar) GetListeners() []any {
	return e.listeners
}

// RegisterEventListener registers an event listener for event type E. If a non-nil error is returned by the listener,
// it will cause the process which emitted the event to fail.
func RegisterEventListener[E protoiface.MessageV1](registrar *EventListenerRegistrar, listener func(context.Context, E) error) {
	registrar.listeners = append(registrar.listeners, listener)
}
