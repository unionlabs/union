// Package event provides a basic API for app modules to emit events.
package event

import (
	"context"

	"cosmossdk.io/core/transaction"
)

// Service represents an event service which can retrieve and set an event manager in a context.
// event.Service is a core API type that should be provided by the runtime module being used to
// build an app via depinject.
type Service interface {
	EventManager(context.Context) Manager
}

// Manager represents an event manager which can emit events.
type Manager interface {
	// Emit emits events represented as a protobuf message (as described in ADR 032).
	//
	// Callers SHOULD assume that these events will not be included in consensus.
	Emit(event transaction.Msg) error

	// EmitKV emits an event based on an event and kv-pair attributes.
	//
	// These events will not be part of consensus and adding, removing or changing these events is
	// not a state-machine breaking change.
	EmitKV(eventType string, attrs ...Attribute) error
}
