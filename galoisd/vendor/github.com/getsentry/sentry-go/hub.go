package sentry

import (
	"context"
	"sync"
	"time"
)

type contextKey int

// Keys used to store values in a Context. Use with Context.Value to access
// values stored by the SDK.
const (
	// HubContextKey is the key used to store the current Hub.
	HubContextKey = contextKey(1)
	// RequestContextKey is the key used to store the current http.Request.
	RequestContextKey = contextKey(2)
)

// defaultMaxBreadcrumbs is the default maximum number of breadcrumbs added to
// an event. Can be overwritten with the maxBreadcrumbs option.
const defaultMaxBreadcrumbs = 30

// maxBreadcrumbs is the absolute maximum number of breadcrumbs added to an
// event. The maxBreadcrumbs option cannot be set higher than this value.
const maxBreadcrumbs = 100

// currentHub is the initial Hub with no Client bound and an empty Scope.
var currentHub = NewHub(nil, NewScope())

// Hub is the central object that manages scopes and clients.
//
// This can be used to capture events and manage the scope.
// The default hub that is available automatically.
//
// In most situations developers do not need to interface the hub. Instead
// toplevel convenience functions are exposed that will automatically dispatch
// to global (CurrentHub) hub.  In some situations this might not be
// possible in which case it might become necessary to manually work with the
// hub. This is for instance the case when working with async code.
type Hub struct {
	mu          sync.RWMutex
	stack       *stack
	lastEventID EventID
}

type layer struct {
	// mu protects concurrent reads and writes to client.
	mu     sync.RWMutex
	client *Client
	// scope is read-only, not protected by mu.
	scope *Scope
}

// Client returns the layer's client. Safe for concurrent use.
func (l *layer) Client() *Client {
	l.mu.RLock()
	defer l.mu.RUnlock()
	return l.client
}

// SetClient sets the layer's client. Safe for concurrent use.
func (l *layer) SetClient(c *Client) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.client = c
}

type stack []*layer

// NewHub returns an instance of a Hub with provided Client and Scope bound.
func NewHub(client *Client, scope *Scope) *Hub {
	hub := Hub{
		stack: &stack{{
			client: client,
			scope:  scope,
		}},
	}
	return &hub
}

// CurrentHub returns an instance of previously initialized Hub stored in the global namespace.
func CurrentHub() *Hub {
	return currentHub
}

// LastEventID returns the ID of the last event (error or message) captured
// through the hub and sent to the underlying transport.
//
// Transactions and events dropped by sampling or event processors do not change
// the last event ID.
//
// LastEventID is a convenience method to cover use cases in which errors are
// captured indirectly and the ID is needed. For example, it can be used as part
// of an HTTP middleware to log the ID of the last error, if any.
//
// For more flexibility, consider instead using the ClientOptions.BeforeSend
// function or event processors.
func (hub *Hub) LastEventID() EventID {
	hub.mu.RLock()
	defer hub.mu.RUnlock()

	return hub.lastEventID
}

// stackTop returns the top layer of the hub stack. Valid hubs always have at
// least one layer, therefore stackTop always return a non-nil pointer.
func (hub *Hub) stackTop() *layer {
	hub.mu.RLock()
	defer hub.mu.RUnlock()

	stack := hub.stack
	stackLen := len(*stack)
	top := (*stack)[stackLen-1]
	return top
}

// Clone returns a copy of the current Hub with top-most scope and client copied over.
func (hub *Hub) Clone() *Hub {
	top := hub.stackTop()
	scope := top.scope
	if scope != nil {
		scope = scope.Clone()
	}
	return NewHub(top.Client(), scope)
}

// Scope returns top-level Scope of the current Hub or nil if no Scope is bound.
func (hub *Hub) Scope() *Scope {
	top := hub.stackTop()
	return top.scope
}

// Client returns top-level Client of the current Hub or nil if no Client is bound.
func (hub *Hub) Client() *Client {
	top := hub.stackTop()
	return top.Client()
}

// PushScope pushes a new scope for the current Hub and reuses previously bound Client.
func (hub *Hub) PushScope() *Scope {
	top := hub.stackTop()

	var scope *Scope
	if top.scope != nil {
		scope = top.scope.Clone()
	} else {
		scope = NewScope()
	}

	hub.mu.Lock()
	defer hub.mu.Unlock()

	*hub.stack = append(*hub.stack, &layer{
		client: top.Client(),
		scope:  scope,
	})

	return scope
}

// PopScope drops the most recent scope.
//
// Calls to PopScope must be coordinated with PushScope. For most cases, using
// WithScope should be more convenient.
//
// Calls to PopScope that do not match previous calls to PushScope are silently
// ignored.
func (hub *Hub) PopScope() {
	hub.mu.Lock()
	defer hub.mu.Unlock()

	stack := *hub.stack
	stackLen := len(stack)
	if stackLen > 1 {
		// Never pop the last item off the stack, the stack should always have
		// at least one item.
		*hub.stack = stack[0 : stackLen-1]
	}
}

// BindClient binds a new Client for the current Hub.
func (hub *Hub) BindClient(client *Client) {
	top := hub.stackTop()
	top.SetClient(client)
}

// WithScope runs f in an isolated temporary scope.
//
// It is useful when extra data should be sent with a single capture call, for
// instance a different level or tags.
//
// The scope passed to f starts as a clone of the current scope and can be
// freely modified without affecting the current scope.
//
// It is a shorthand for PushScope followed by PopScope.
func (hub *Hub) WithScope(f func(scope *Scope)) {
	scope := hub.PushScope()
	defer hub.PopScope()
	f(scope)
}

// ConfigureScope runs f in the current scope.
//
// It is useful to set data that applies to all events that share the current
// scope.
//
// Modifying the scope affects all references to the current scope.
//
// See also WithScope for making isolated temporary changes.
func (hub *Hub) ConfigureScope(f func(scope *Scope)) {
	scope := hub.Scope()
	f(scope)
}

// CaptureEvent calls the method of a same name on currently bound Client instance
// passing it a top-level Scope.
// Returns EventID if successfully, or nil if there's no Scope or Client available.
func (hub *Hub) CaptureEvent(event *Event) *EventID {
	client, scope := hub.Client(), hub.Scope()
	if client == nil || scope == nil {
		return nil
	}
	eventID := client.CaptureEvent(event, nil, scope)

	if event.Type != transactionType && eventID != nil {
		hub.mu.Lock()
		hub.lastEventID = *eventID
		hub.mu.Unlock()
	}
	return eventID
}

// CaptureMessage calls the method of a same name on currently bound Client instance
// passing it a top-level Scope.
// Returns EventID if successfully, or nil if there's no Scope or Client available.
func (hub *Hub) CaptureMessage(message string) *EventID {
	client, scope := hub.Client(), hub.Scope()
	if client == nil || scope == nil {
		return nil
	}
	eventID := client.CaptureMessage(message, nil, scope)

	if eventID != nil {
		hub.mu.Lock()
		hub.lastEventID = *eventID
		hub.mu.Unlock()
	}
	return eventID
}

// CaptureException calls the method of a same name on currently bound Client instance
// passing it a top-level Scope.
// Returns EventID if successfully, or nil if there's no Scope or Client available.
func (hub *Hub) CaptureException(exception error) *EventID {
	client, scope := hub.Client(), hub.Scope()
	if client == nil || scope == nil {
		return nil
	}
	eventID := client.CaptureException(exception, &EventHint{OriginalException: exception}, scope)

	if eventID != nil {
		hub.mu.Lock()
		hub.lastEventID = *eventID
		hub.mu.Unlock()
	}
	return eventID
}

// CaptureCheckIn calls the method of the same name on currently bound Client instance
// passing it a top-level Scope.
// Returns CheckInID if the check-in was captured successfully, or nil otherwise.
func (hub *Hub) CaptureCheckIn(checkIn *CheckIn, monitorConfig *MonitorConfig) *EventID {
	client, scope := hub.Client(), hub.Scope()
	if client == nil {
		return nil
	}

	return client.CaptureCheckIn(checkIn, monitorConfig, scope)
}

// AddBreadcrumb records a new breadcrumb.
//
// The total number of breadcrumbs that can be recorded are limited by the
// configuration on the client.
func (hub *Hub) AddBreadcrumb(breadcrumb *Breadcrumb, hint *BreadcrumbHint) {
	client := hub.Client()

	// If there's no client, just store it on the scope straight away
	if client == nil {
		hub.Scope().AddBreadcrumb(breadcrumb, maxBreadcrumbs)
		return
	}

	max := client.options.MaxBreadcrumbs
	if max < 0 {
		return
	}

	if client.options.BeforeBreadcrumb != nil {
		if hint == nil {
			hint = &BreadcrumbHint{}
		}
		if breadcrumb = client.options.BeforeBreadcrumb(breadcrumb, hint); breadcrumb == nil {
			Logger.Println("breadcrumb dropped due to BeforeBreadcrumb callback.")
			return
		}
	}

	if max == 0 {
		max = defaultMaxBreadcrumbs
	} else if max > maxBreadcrumbs {
		max = maxBreadcrumbs
	}

	hub.Scope().AddBreadcrumb(breadcrumb, max)
}

// Recover calls the method of a same name on currently bound Client instance
// passing it a top-level Scope.
// Returns EventID if successfully, or nil if there's no Scope or Client available.
func (hub *Hub) Recover(err interface{}) *EventID {
	if err == nil {
		err = recover()
	}
	client, scope := hub.Client(), hub.Scope()
	if client == nil || scope == nil {
		return nil
	}
	return client.Recover(err, &EventHint{RecoveredException: err}, scope)
}

// RecoverWithContext calls the method of a same name on currently bound Client instance
// passing it a top-level Scope.
// Returns EventID if successfully, or nil if there's no Scope or Client available.
func (hub *Hub) RecoverWithContext(ctx context.Context, err interface{}) *EventID {
	if err == nil {
		err = recover()
	}
	client, scope := hub.Client(), hub.Scope()
	if client == nil || scope == nil {
		return nil
	}
	return client.RecoverWithContext(ctx, err, &EventHint{RecoveredException: err}, scope)
}

// Flush waits until the underlying Transport sends any buffered events to the
// Sentry server, blocking for at most the given timeout. It returns false if
// the timeout was reached. In that case, some events may not have been sent.
//
// Flush should be called before terminating the program to avoid
// unintentionally dropping events.
//
// Do not call Flush indiscriminately after every call to CaptureEvent,
// CaptureException or CaptureMessage. Instead, to have the SDK send events over
// the network synchronously, configure it to use the HTTPSyncTransport in the
// call to Init.
func (hub *Hub) Flush(timeout time.Duration) bool {
	client := hub.Client()

	if client == nil {
		return false
	}

	return client.Flush(timeout)
}

// HasHubOnContext checks whether Hub instance is bound to a given Context struct.
func HasHubOnContext(ctx context.Context) bool {
	_, ok := ctx.Value(HubContextKey).(*Hub)
	return ok
}

// GetHubFromContext tries to retrieve Hub instance from the given Context struct
// or return nil if one is not found.
func GetHubFromContext(ctx context.Context) *Hub {
	if hub, ok := ctx.Value(HubContextKey).(*Hub); ok {
		return hub
	}
	return nil
}

// hubFromContext returns either a hub stored in the context or the current hub.
// The return value is guaranteed to be non-nil, unlike GetHubFromContext.
func hubFromContext(ctx context.Context) *Hub {
	if hub, ok := ctx.Value(HubContextKey).(*Hub); ok {
		return hub
	}
	return currentHub
}

// SetHubOnContext stores given Hub instance on the Context struct and returns a new Context.
func SetHubOnContext(ctx context.Context, hub *Hub) context.Context {
	return context.WithValue(ctx, HubContextKey, hub)
}
