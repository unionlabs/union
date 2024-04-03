package sentry

import (
	"bytes"
	"io"
	"net/http"
	"sync"
	"time"
)

// Scope holds contextual data for the current scope.
//
// The scope is an object that can cloned efficiently and stores data that is
// locally relevant to an event. For instance the scope will hold recorded
// breadcrumbs and similar information.
//
// The scope can be interacted with in two ways. First, the scope is routinely
// updated with information by functions such as AddBreadcrumb which will modify
// the current scope. Second, the current scope can be configured through the
// ConfigureScope function or Hub method of the same name.
//
// The scope is meant to be modified but not inspected directly. When preparing
// an event for reporting, the current client adds information from the current
// scope into the event.
type Scope struct {
	mu          sync.RWMutex
	breadcrumbs []*Breadcrumb
	attachments []*Attachment
	user        User
	tags        map[string]string
	contexts    map[string]Context
	extra       map[string]interface{}
	fingerprint []string
	level       Level
	request     *http.Request
	// requestBody holds a reference to the original request.Body.
	requestBody interface {
		// Bytes returns bytes from the original body, lazily buffered as the
		// original body is read.
		Bytes() []byte
		// Overflow returns true if the body is larger than the maximum buffer
		// size.
		Overflow() bool
	}
	eventProcessors []EventProcessor
}

// NewScope creates a new Scope.
func NewScope() *Scope {
	scope := Scope{
		breadcrumbs: make([]*Breadcrumb, 0),
		attachments: make([]*Attachment, 0),
		tags:        make(map[string]string),
		contexts:    make(map[string]Context),
		extra:       make(map[string]interface{}),
		fingerprint: make([]string, 0),
	}

	return &scope
}

// AddBreadcrumb adds new breadcrumb to the current scope
// and optionally throws the old one if limit is reached.
func (scope *Scope) AddBreadcrumb(breadcrumb *Breadcrumb, limit int) {
	if breadcrumb.Timestamp.IsZero() {
		breadcrumb.Timestamp = time.Now()
	}

	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.breadcrumbs = append(scope.breadcrumbs, breadcrumb)
	if len(scope.breadcrumbs) > limit {
		scope.breadcrumbs = scope.breadcrumbs[1 : limit+1]
	}
}

// ClearBreadcrumbs clears all breadcrumbs from the current scope.
func (scope *Scope) ClearBreadcrumbs() {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.breadcrumbs = []*Breadcrumb{}
}

// AddAttachment adds new attachment to the current scope.
func (scope *Scope) AddAttachment(attachment *Attachment) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.attachments = append(scope.attachments, attachment)
}

// ClearAttachments clears all attachments from the current scope.
func (scope *Scope) ClearAttachments() {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.attachments = []*Attachment{}
}

// SetUser sets the user for the current scope.
func (scope *Scope) SetUser(user User) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.user = user
}

// SetRequest sets the request for the current scope.
func (scope *Scope) SetRequest(r *http.Request) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.request = r

	if r == nil {
		return
	}

	// Don't buffer request body if we know it is oversized.
	if r.ContentLength > maxRequestBodyBytes {
		return
	}
	// Don't buffer if there is no body.
	if r.Body == nil || r.Body == http.NoBody {
		return
	}
	buf := &limitedBuffer{Capacity: maxRequestBodyBytes}
	r.Body = readCloser{
		Reader: io.TeeReader(r.Body, buf),
		Closer: r.Body,
	}
	scope.requestBody = buf
}

// SetRequestBody sets the request body for the current scope.
//
// This method should only be called when the body bytes are already available
// in memory. Typically, the request body is buffered lazily from the
// Request.Body from SetRequest.
func (scope *Scope) SetRequestBody(b []byte) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	capacity := maxRequestBodyBytes
	overflow := false
	if len(b) > capacity {
		overflow = true
		b = b[:capacity]
	}
	scope.requestBody = &limitedBuffer{
		Capacity: capacity,
		Buffer:   *bytes.NewBuffer(b),
		overflow: overflow,
	}
}

// maxRequestBodyBytes is the default maximum request body size to send to
// Sentry.
const maxRequestBodyBytes = 10 * 1024

// A limitedBuffer is like a bytes.Buffer, but limited to store at most Capacity
// bytes. Any writes past the capacity are silently discarded, similar to
// io.Discard.
type limitedBuffer struct {
	Capacity int

	bytes.Buffer
	overflow bool
}

// Write implements io.Writer.
func (b *limitedBuffer) Write(p []byte) (n int, err error) {
	// Silently ignore writes after overflow.
	if b.overflow {
		return len(p), nil
	}
	left := b.Capacity - b.Len()
	if left < 0 {
		left = 0
	}
	if len(p) > left {
		b.overflow = true
		p = p[:left]
	}
	return b.Buffer.Write(p)
}

// Overflow returns true if the limitedBuffer discarded bytes written to it.
func (b *limitedBuffer) Overflow() bool {
	return b.overflow
}

// readCloser combines an io.Reader and an io.Closer to implement io.ReadCloser.
type readCloser struct {
	io.Reader
	io.Closer
}

// SetTag adds a tag to the current scope.
func (scope *Scope) SetTag(key, value string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.tags[key] = value
}

// SetTags assigns multiple tags to the current scope.
func (scope *Scope) SetTags(tags map[string]string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	for k, v := range tags {
		scope.tags[k] = v
	}
}

// RemoveTag removes a tag from the current scope.
func (scope *Scope) RemoveTag(key string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	delete(scope.tags, key)
}

// SetContext adds a context to the current scope.
func (scope *Scope) SetContext(key string, value Context) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.contexts[key] = value
}

// SetContexts assigns multiple contexts to the current scope.
func (scope *Scope) SetContexts(contexts map[string]Context) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	for k, v := range contexts {
		scope.contexts[k] = v
	}
}

// RemoveContext removes a context from the current scope.
func (scope *Scope) RemoveContext(key string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	delete(scope.contexts, key)
}

// SetExtra adds an extra to the current scope.
func (scope *Scope) SetExtra(key string, value interface{}) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.extra[key] = value
}

// SetExtras assigns multiple extras to the current scope.
func (scope *Scope) SetExtras(extra map[string]interface{}) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	for k, v := range extra {
		scope.extra[k] = v
	}
}

// RemoveExtra removes a extra from the current scope.
func (scope *Scope) RemoveExtra(key string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	delete(scope.extra, key)
}

// SetFingerprint sets new fingerprint for the current scope.
func (scope *Scope) SetFingerprint(fingerprint []string) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.fingerprint = fingerprint
}

// SetLevel sets new level for the current scope.
func (scope *Scope) SetLevel(level Level) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.level = level
}

// Clone returns a copy of the current scope with all data copied over.
func (scope *Scope) Clone() *Scope {
	scope.mu.RLock()
	defer scope.mu.RUnlock()

	clone := NewScope()
	clone.user = scope.user
	clone.breadcrumbs = make([]*Breadcrumb, len(scope.breadcrumbs))
	copy(clone.breadcrumbs, scope.breadcrumbs)
	clone.attachments = make([]*Attachment, len(scope.attachments))
	copy(clone.attachments, scope.attachments)
	for key, value := range scope.tags {
		clone.tags[key] = value
	}
	for key, value := range scope.contexts {
		clone.contexts[key] = cloneContext(value)
	}
	for key, value := range scope.extra {
		clone.extra[key] = value
	}
	clone.fingerprint = make([]string, len(scope.fingerprint))
	copy(clone.fingerprint, scope.fingerprint)
	clone.level = scope.level
	clone.request = scope.request
	clone.requestBody = scope.requestBody
	clone.eventProcessors = scope.eventProcessors
	return clone
}

// Clear removes the data from the current scope. Not safe for concurrent use.
func (scope *Scope) Clear() {
	*scope = *NewScope()
}

// AddEventProcessor adds an event processor to the current scope.
func (scope *Scope) AddEventProcessor(processor EventProcessor) {
	scope.mu.Lock()
	defer scope.mu.Unlock()

	scope.eventProcessors = append(scope.eventProcessors, processor)
}

// ApplyToEvent takes the data from the current scope and attaches it to the event.
func (scope *Scope) ApplyToEvent(event *Event, hint *EventHint) *Event {
	scope.mu.RLock()
	defer scope.mu.RUnlock()

	if len(scope.breadcrumbs) > 0 {
		event.Breadcrumbs = append(event.Breadcrumbs, scope.breadcrumbs...)
	}

	if len(scope.attachments) > 0 {
		event.Attachments = append(event.Attachments, scope.attachments...)
	}

	if len(scope.tags) > 0 {
		if event.Tags == nil {
			event.Tags = make(map[string]string, len(scope.tags))
		}

		for key, value := range scope.tags {
			event.Tags[key] = value
		}
	}

	if len(scope.contexts) > 0 {
		if event.Contexts == nil {
			event.Contexts = make(map[string]Context)
		}

		for key, value := range scope.contexts {
			if key == "trace" && event.Type == transactionType {
				// Do not override trace context of
				// transactions, otherwise it breaks the
				// transaction event representation.
				// For error events, the trace context is used
				// to link errors and traces/spans in Sentry.
				continue
			}

			// Ensure we are not overwriting event fields
			if _, ok := event.Contexts[key]; !ok {
				event.Contexts[key] = cloneContext(value)
			}
		}
	}

	if len(scope.extra) > 0 {
		if event.Extra == nil {
			event.Extra = make(map[string]interface{}, len(scope.extra))
		}

		for key, value := range scope.extra {
			event.Extra[key] = value
		}
	}

	if event.User.IsEmpty() {
		event.User = scope.user
	}

	if len(event.Fingerprint) == 0 {
		event.Fingerprint = append(event.Fingerprint, scope.fingerprint...)
	}

	if scope.level != "" {
		event.Level = scope.level
	}

	if event.Request == nil && scope.request != nil {
		event.Request = NewRequest(scope.request)
		// NOTE: The SDK does not attempt to send partial request body data.
		//
		// The reason being that Sentry's ingest pipeline and UI are optimized
		// to show structured data. Additionally, tooling around PII scrubbing
		// relies on structured data; truncated request bodies would create
		// invalid payloads that are more prone to leaking PII data.
		//
		// Users can still send more data along their events if they want to,
		// for example using Event.Extra.
		if scope.requestBody != nil && !scope.requestBody.Overflow() {
			event.Request.Data = string(scope.requestBody.Bytes())
		}
	}

	for _, processor := range scope.eventProcessors {
		id := event.EventID
		event = processor(event, hint)
		if event == nil {
			Logger.Printf("Event dropped by one of the Scope EventProcessors: %s\n", id)
			return nil
		}
	}

	return event
}

// cloneContext returns a new context with keys and values copied from the passed one.
//
// Note: a new Context (map) is returned, but the function does NOT do
// a proper deep copy: if some context values are pointer types (e.g. maps),
// they won't be properly copied.
func cloneContext(c Context) Context {
	res := Context{}
	for k, v := range c {
		res[k] = v
	}
	return res
}
