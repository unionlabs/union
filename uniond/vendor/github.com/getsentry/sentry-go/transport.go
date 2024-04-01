package sentry

import (
	"bytes"
	"crypto/tls"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sync"
	"time"

	"github.com/getsentry/sentry-go/internal/ratelimit"
)

const defaultBufferSize = 30
const defaultTimeout = time.Second * 30

// maxDrainResponseBytes is the maximum number of bytes that transport
// implementations will read from response bodies when draining them.
//
// Sentry's ingestion API responses are typically short and the SDK doesn't need
// the contents of the response body. However, the net/http HTTP client requires
// response bodies to be fully drained (and closed) for TCP keep-alive to work.
//
// maxDrainResponseBytes strikes a balance between reading too much data (if the
// server is misbehaving) and reusing TCP connections.
const maxDrainResponseBytes = 16 << 10

// Transport is used by the Client to deliver events to remote server.
type Transport interface {
	Flush(timeout time.Duration) bool
	Configure(options ClientOptions)
	SendEvent(event *Event)
}

func getProxyConfig(options ClientOptions) func(*http.Request) (*url.URL, error) {
	if options.HTTPSProxy != "" {
		return func(*http.Request) (*url.URL, error) {
			return url.Parse(options.HTTPSProxy)
		}
	}

	if options.HTTPProxy != "" {
		return func(*http.Request) (*url.URL, error) {
			return url.Parse(options.HTTPProxy)
		}
	}

	return http.ProxyFromEnvironment
}

func getTLSConfig(options ClientOptions) *tls.Config {
	if options.CaCerts != nil {
		// #nosec G402 -- We should be using `MinVersion: tls.VersionTLS12`,
		// 				 but we don't want to break peoples code without the major bump.
		return &tls.Config{
			RootCAs: options.CaCerts,
		}
	}

	return nil
}

func getRequestBodyFromEvent(event *Event) []byte {
	body, err := json.Marshal(event)
	if err == nil {
		return body
	}

	msg := fmt.Sprintf("Could not encode original event as JSON. "+
		"Succeeded by removing Breadcrumbs, Contexts and Extra. "+
		"Please verify the data you attach to the scope. "+
		"Error: %s", err)
	// Try to serialize the event, with all the contextual data that allows for interface{} stripped.
	event.Breadcrumbs = nil
	event.Contexts = nil
	event.Extra = map[string]interface{}{
		"info": msg,
	}
	body, err = json.Marshal(event)
	if err == nil {
		Logger.Println(msg)
		return body
	}

	// This should _only_ happen when Event.Exception[0].Stacktrace.Frames[0].Vars is unserializable
	// Which won't ever happen, as we don't use it now (although it's the part of public interface accepted by Sentry)
	// Juuust in case something, somehow goes utterly wrong.
	Logger.Println("Event couldn't be marshaled, even with stripped contextual data. Skipping delivery. " +
		"Please notify the SDK owners with possibly broken payload.")
	return nil
}

func encodeAttachment(enc *json.Encoder, b io.Writer, attachment *Attachment) error {
	// Attachment header
	err := enc.Encode(struct {
		Type        string `json:"type"`
		Length      int    `json:"length"`
		Filename    string `json:"filename"`
		ContentType string `json:"content_type,omitempty"`
	}{
		Type:        "attachment",
		Length:      len(attachment.Payload),
		Filename:    attachment.Filename,
		ContentType: attachment.ContentType,
	})
	if err != nil {
		return err
	}

	// Attachment payload
	if _, err = b.Write(attachment.Payload); err != nil {
		return err
	}

	// "Envelopes should be terminated with a trailing newline."
	//
	// [1]: https://develop.sentry.dev/sdk/envelopes/#envelopes
	if _, err := b.Write([]byte("\n")); err != nil {
		return err
	}

	return nil
}

func encodeEnvelopeItem(enc *json.Encoder, itemType string, body json.RawMessage) error {
	// Item header
	err := enc.Encode(struct {
		Type   string `json:"type"`
		Length int    `json:"length"`
	}{
		Type:   itemType,
		Length: len(body),
	})
	if err == nil {
		// payload
		err = enc.Encode(body)
	}
	return err
}

func envelopeFromBody(event *Event, dsn *Dsn, sentAt time.Time, body json.RawMessage) (*bytes.Buffer, error) {
	var b bytes.Buffer
	enc := json.NewEncoder(&b)

	// Construct the trace envelope header
	var trace = map[string]string{}
	if dsc := event.sdkMetaData.dsc; dsc.HasEntries() {
		for k, v := range dsc.Entries {
			trace[k] = v
		}
	}

	// Envelope header
	err := enc.Encode(struct {
		EventID EventID           `json:"event_id"`
		SentAt  time.Time         `json:"sent_at"`
		Dsn     string            `json:"dsn"`
		Sdk     map[string]string `json:"sdk"`
		Trace   map[string]string `json:"trace,omitempty"`
	}{
		EventID: event.EventID,
		SentAt:  sentAt,
		Trace:   trace,
		Dsn:     dsn.String(),
		Sdk: map[string]string{
			"name":    event.Sdk.Name,
			"version": event.Sdk.Version,
		},
	})
	if err != nil {
		return nil, err
	}

	if event.Type == transactionType || event.Type == checkInType {
		err = encodeEnvelopeItem(enc, event.Type, body)
	} else {
		err = encodeEnvelopeItem(enc, eventType, body)
	}
	if err != nil {
		return nil, err
	}

	// Attachments
	for _, attachment := range event.Attachments {
		if err := encodeAttachment(enc, &b, attachment); err != nil {
			return nil, err
		}
	}

	// Profile data
	if event.sdkMetaData.transactionProfile != nil {
		body, err = json.Marshal(event.sdkMetaData.transactionProfile)
		if err != nil {
			return nil, err
		}
		err = encodeEnvelopeItem(enc, profileType, body)
		if err != nil {
			return nil, err
		}
	}

	return &b, nil
}

func getRequestFromEvent(event *Event, dsn *Dsn) (r *http.Request, err error) {
	defer func() {
		if r != nil {
			r.Header.Set("User-Agent", fmt.Sprintf("%s/%s", event.Sdk.Name, event.Sdk.Version))
			r.Header.Set("Content-Type", "application/x-sentry-envelope")

			auth := fmt.Sprintf("Sentry sentry_version=%s, "+
				"sentry_client=%s/%s, sentry_key=%s", apiVersion, event.Sdk.Name, event.Sdk.Version, dsn.publicKey)

			// The key sentry_secret is effectively deprecated and no longer needs to be set.
			// However, since it was required in older self-hosted versions,
			// it should still passed through to Sentry if set.
			if dsn.secretKey != "" {
				auth = fmt.Sprintf("%s, sentry_secret=%s", auth, dsn.secretKey)
			}

			r.Header.Set("X-Sentry-Auth", auth)
		}
	}()
	body := getRequestBodyFromEvent(event)
	if body == nil {
		return nil, errors.New("event could not be marshaled")
	}
	envelope, err := envelopeFromBody(event, dsn, time.Now(), body)
	if err != nil {
		return nil, err
	}
	return http.NewRequest(
		http.MethodPost,
		dsn.GetAPIURL().String(),
		envelope,
	)
}

func categoryFor(eventType string) ratelimit.Category {
	switch eventType {
	case "":
		return ratelimit.CategoryError
	case transactionType:
		return ratelimit.CategoryTransaction
	default:
		return ratelimit.Category(eventType)
	}
}

// ================================
// HTTPTransport
// ================================

// A batch groups items that are processed sequentially.
type batch struct {
	items   chan batchItem
	started chan struct{} // closed to signal items started to be worked on
	done    chan struct{} // closed to signal completion of all items
}

type batchItem struct {
	request  *http.Request
	category ratelimit.Category
}

// HTTPTransport is the default, non-blocking, implementation of Transport.
//
// Clients using this transport will enqueue requests in a buffer and return to
// the caller before any network communication has happened. Requests are sent
// to Sentry sequentially from a background goroutine.
type HTTPTransport struct {
	dsn       *Dsn
	client    *http.Client
	transport http.RoundTripper

	// buffer is a channel of batches. Calling Flush terminates work on the
	// current in-flight items and starts a new batch for subsequent events.
	buffer chan batch

	start sync.Once

	// Size of the transport buffer. Defaults to 30.
	BufferSize int
	// HTTP Client request timeout. Defaults to 30 seconds.
	Timeout time.Duration

	mu     sync.RWMutex
	limits ratelimit.Map
}

// NewHTTPTransport returns a new pre-configured instance of HTTPTransport.
func NewHTTPTransport() *HTTPTransport {
	transport := HTTPTransport{
		BufferSize: defaultBufferSize,
		Timeout:    defaultTimeout,
		limits:     make(ratelimit.Map),
	}
	return &transport
}

// Configure is called by the Client itself, providing it it's own ClientOptions.
func (t *HTTPTransport) Configure(options ClientOptions) {
	dsn, err := NewDsn(options.Dsn)
	if err != nil {
		Logger.Printf("%v\n", err)
		return
	}
	t.dsn = dsn

	// A buffered channel with capacity 1 works like a mutex, ensuring only one
	// goroutine can access the current batch at a given time. Access is
	// synchronized by reading from and writing to the channel.
	t.buffer = make(chan batch, 1)
	t.buffer <- batch{
		items:   make(chan batchItem, t.BufferSize),
		started: make(chan struct{}),
		done:    make(chan struct{}),
	}

	if options.HTTPTransport != nil {
		t.transport = options.HTTPTransport
	} else {
		t.transport = &http.Transport{
			Proxy:           getProxyConfig(options),
			TLSClientConfig: getTLSConfig(options),
		}
	}

	if options.HTTPClient != nil {
		t.client = options.HTTPClient
	} else {
		t.client = &http.Client{
			Transport: t.transport,
			Timeout:   t.Timeout,
		}
	}

	t.start.Do(func() {
		go t.worker()
	})
}

// SendEvent assembles a new packet out of Event and sends it to remote server.
func (t *HTTPTransport) SendEvent(event *Event) {
	if t.dsn == nil {
		return
	}

	category := categoryFor(event.Type)

	if t.disabled(category) {
		return
	}

	request, err := getRequestFromEvent(event, t.dsn)
	if err != nil {
		return
	}

	// <-t.buffer is equivalent to acquiring a lock to access the current batch.
	// A few lines below, t.buffer <- b releases the lock.
	//
	// The lock must be held during the select block below to guarantee that
	// b.items is not closed while trying to send to it. Remember that sending
	// on a closed channel panics.
	//
	// Note that the select block takes a bounded amount of CPU time because of
	// the default case that is executed if sending on b.items would block. That
	// is, the event is dropped if it cannot be sent immediately to the b.items
	// channel (used as a queue).
	b := <-t.buffer

	select {
	case b.items <- batchItem{
		request:  request,
		category: category,
	}:
		var eventType string
		if event.Type == transactionType {
			eventType = "transaction"
		} else {
			eventType = fmt.Sprintf("%s event", event.Level)
		}
		Logger.Printf(
			"Sending %s [%s] to %s project: %s",
			eventType,
			event.EventID,
			t.dsn.host,
			t.dsn.projectID,
		)
	default:
		Logger.Println("Event dropped due to transport buffer being full.")
	}

	t.buffer <- b
}

// Flush waits until any buffered events are sent to the Sentry server, blocking
// for at most the given timeout. It returns false if the timeout was reached.
// In that case, some events may not have been sent.
//
// Flush should be called before terminating the program to avoid
// unintentionally dropping events.
//
// Do not call Flush indiscriminately after every call to SendEvent. Instead, to
// have the SDK send events over the network synchronously, configure it to use
// the HTTPSyncTransport in the call to Init.
func (t *HTTPTransport) Flush(timeout time.Duration) bool {
	toolate := time.After(timeout)

	// Wait until processing the current batch has started or the timeout.
	//
	// We must wait until the worker has seen the current batch, because it is
	// the only way b.done will be closed. If we do not wait, there is a
	// possible execution flow in which b.done is never closed, and the only way
	// out of Flush would be waiting for the timeout, which is undesired.
	var b batch
	for {
		select {
		case b = <-t.buffer:
			select {
			case <-b.started:
				goto started
			default:
				t.buffer <- b
			}
		case <-toolate:
			goto fail
		}
	}

started:
	// Signal that there won't be any more items in this batch, so that the
	// worker inner loop can end.
	close(b.items)
	// Start a new batch for subsequent events.
	t.buffer <- batch{
		items:   make(chan batchItem, t.BufferSize),
		started: make(chan struct{}),
		done:    make(chan struct{}),
	}

	// Wait until the current batch is done or the timeout.
	select {
	case <-b.done:
		Logger.Println("Buffer flushed successfully.")
		return true
	case <-toolate:
		goto fail
	}

fail:
	Logger.Println("Buffer flushing reached the timeout.")
	return false
}

func (t *HTTPTransport) worker() {
	for b := range t.buffer {
		// Signal that processing of the current batch has started.
		close(b.started)

		// Return the batch to the buffer so that other goroutines can use it.
		// Equivalent to releasing a lock.
		t.buffer <- b

		// Process all batch items.
		for item := range b.items {
			if t.disabled(item.category) {
				continue
			}

			response, err := t.client.Do(item.request)
			if err != nil {
				Logger.Printf("There was an issue with sending an event: %v", err)
				continue
			}
			t.mu.Lock()
			t.limits.Merge(ratelimit.FromResponse(response))
			t.mu.Unlock()
			// Drain body up to a limit and close it, allowing the
			// transport to reuse TCP connections.
			_, _ = io.CopyN(io.Discard, response.Body, maxDrainResponseBytes)
			response.Body.Close()
		}

		// Signal that processing of the batch is done.
		close(b.done)
	}
}

func (t *HTTPTransport) disabled(c ratelimit.Category) bool {
	t.mu.RLock()
	defer t.mu.RUnlock()
	disabled := t.limits.IsRateLimited(c)
	if disabled {
		Logger.Printf("Too many requests for %q, backing off till: %v", c, t.limits.Deadline(c))
	}
	return disabled
}

// ================================
// HTTPSyncTransport
// ================================

// HTTPSyncTransport is a blocking implementation of Transport.
//
// Clients using this transport will send requests to Sentry sequentially and
// block until a response is returned.
//
// The blocking behavior is useful in a limited set of use cases. For example,
// use it when deploying code to a Function as a Service ("Serverless")
// platform, where any work happening in a background goroutine is not
// guaranteed to execute.
//
// For most cases, prefer HTTPTransport.
type HTTPSyncTransport struct {
	dsn       *Dsn
	client    *http.Client
	transport http.RoundTripper

	mu     sync.Mutex
	limits ratelimit.Map

	// HTTP Client request timeout. Defaults to 30 seconds.
	Timeout time.Duration
}

// NewHTTPSyncTransport returns a new pre-configured instance of HTTPSyncTransport.
func NewHTTPSyncTransport() *HTTPSyncTransport {
	transport := HTTPSyncTransport{
		Timeout: defaultTimeout,
		limits:  make(ratelimit.Map),
	}

	return &transport
}

// Configure is called by the Client itself, providing it it's own ClientOptions.
func (t *HTTPSyncTransport) Configure(options ClientOptions) {
	dsn, err := NewDsn(options.Dsn)
	if err != nil {
		Logger.Printf("%v\n", err)
		return
	}
	t.dsn = dsn

	if options.HTTPTransport != nil {
		t.transport = options.HTTPTransport
	} else {
		t.transport = &http.Transport{
			Proxy:           getProxyConfig(options),
			TLSClientConfig: getTLSConfig(options),
		}
	}

	if options.HTTPClient != nil {
		t.client = options.HTTPClient
	} else {
		t.client = &http.Client{
			Transport: t.transport,
			Timeout:   t.Timeout,
		}
	}
}

// SendEvent assembles a new packet out of Event and sends it to remote server.
func (t *HTTPSyncTransport) SendEvent(event *Event) {
	if t.dsn == nil {
		return
	}

	if t.disabled(categoryFor(event.Type)) {
		return
	}

	request, err := getRequestFromEvent(event, t.dsn)
	if err != nil {
		return
	}

	var eventType string
	if event.Type == transactionType {
		eventType = "transaction"
	} else {
		eventType = fmt.Sprintf("%s event", event.Level)
	}
	Logger.Printf(
		"Sending %s [%s] to %s project: %s",
		eventType,
		event.EventID,
		t.dsn.host,
		t.dsn.projectID,
	)

	response, err := t.client.Do(request)
	if err != nil {
		Logger.Printf("There was an issue with sending an event: %v", err)
		return
	}
	t.mu.Lock()
	t.limits.Merge(ratelimit.FromResponse(response))
	t.mu.Unlock()

	// Drain body up to a limit and close it, allowing the
	// transport to reuse TCP connections.
	_, _ = io.CopyN(io.Discard, response.Body, maxDrainResponseBytes)
	response.Body.Close()
}

// Flush is a no-op for HTTPSyncTransport. It always returns true immediately.
func (t *HTTPSyncTransport) Flush(_ time.Duration) bool {
	return true
}

func (t *HTTPSyncTransport) disabled(c ratelimit.Category) bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	disabled := t.limits.IsRateLimited(c)
	if disabled {
		Logger.Printf("Too many requests for %q, backing off till: %v", c, t.limits.Deadline(c))
	}
	return disabled
}

// ================================
// noopTransport
// ================================

// noopTransport is an implementation of Transport interface which drops all the events.
// Only used internally when an empty DSN is provided, which effectively disables the SDK.
type noopTransport struct{}

var _ Transport = noopTransport{}

func (noopTransport) Configure(ClientOptions) {
	Logger.Println("Sentry client initialized with an empty DSN. Using noopTransport. No events will be delivered.")
}

func (noopTransport) SendEvent(*Event) {
	Logger.Println("Event dropped due to noopTransport usage.")
}

func (noopTransport) Flush(time.Duration) bool {
	return true
}
