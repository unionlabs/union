package sentry

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/http/httptrace"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/getsentry/sentry-go/internal/testutils"
	"github.com/google/go-cmp/cmp"
)

type unserializableType struct {
	UnsupportedField func()
}

const (
	basicEvent                         = `{"message":"mkey","sdk":{},"user":{}}`
	enhancedEventInvalidBreadcrumb     = `{"extra":{"info":"Could not encode original event as JSON. Succeeded by removing Breadcrumbs, Contexts and Extra. Please verify the data you attach to the scope. Error: json: error calling MarshalJSON for type *sentry.Event: json: error calling MarshalJSON for type *sentry.Breadcrumb: json: unsupported type: func()"},"message":"mkey","sdk":{},"user":{}}`
	enhancedEventInvalidContextOrExtra = `{"extra":{"info":"Could not encode original event as JSON. Succeeded by removing Breadcrumbs, Contexts and Extra. Please verify the data you attach to the scope. Error: json: error calling MarshalJSON for type *sentry.Event: json: unsupported type: func()"},"message":"mkey","sdk":{},"user":{}}`
)

func TestGetRequestBodyFromEventValid(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Message: "mkey",
	})

	got := string(body)
	want := basicEvent

	if got != want {
		t.Errorf("expected different shape of body. \ngot: %s\nwant: %s", got, want)
	}
}

func TestGetRequestBodyFromEventInvalidBreadcrumbsField(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Message: "mkey",
		Breadcrumbs: []*Breadcrumb{{
			Data: map[string]interface{}{
				"wat": unserializableType{},
			},
		}},
	})

	got := string(body)
	want := enhancedEventInvalidBreadcrumb

	if got != want {
		t.Errorf("expected different shape of body. \ngot: %s\nwant: %s", got, want)
	}
}

func TestGetRequestBodyFromEventInvalidExtraField(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Message: "mkey",
		Extra: map[string]interface{}{
			"wat": unserializableType{},
		},
	})

	got := string(body)
	want := enhancedEventInvalidContextOrExtra

	if got != want {
		t.Errorf("expected different shape of body. \ngot: %s\nwant: %s", got, want)
	}
}

func TestGetRequestBodyFromEventInvalidContextField(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Message: "mkey",
		Contexts: map[string]Context{
			"wat": {"key": unserializableType{}},
		},
	})

	got := string(body)
	want := enhancedEventInvalidContextOrExtra

	if got != want {
		t.Errorf("expected different shape of body. \ngot: %s\nwant: %s", got, want)
	}
}

func TestGetRequestBodyFromEventMultipleInvalidFields(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Message: "mkey",
		Breadcrumbs: []*Breadcrumb{{
			Data: map[string]interface{}{
				"wat": unserializableType{},
			},
		}},
		Extra: map[string]interface{}{
			"wat": unserializableType{},
		},
		Contexts: map[string]Context{
			"wat": {"key": unserializableType{}},
		},
	})

	got := string(body)
	want := enhancedEventInvalidBreadcrumb

	if got != want {
		t.Errorf("expected different shape of body. \ngot: %s\nwant: %s", got, want)
	}
}

func TestGetRequestBodyFromEventCompletelyInvalid(t *testing.T) {
	body := getRequestBodyFromEvent(&Event{
		Exception: []Exception{{
			Stacktrace: &Stacktrace{
				Frames: []Frame{{
					Vars: map[string]interface{}{
						"wat": unserializableType{},
					},
				}},
			},
		}},
	})

	if body != nil {
		t.Error("expected body to be nil")
	}
}

func newTestEvent(eventType string) *Event {
	event := NewEvent()
	event.Type = eventType
	event.EventID = "b81c5be4d31e48959103a1f878a1efcb"
	event.Sdk = SdkInfo{
		Name:    "sentry.go",
		Version: "0.0.1",
	}
	return event
}

func newTestDSN(t *testing.T) *Dsn {
	dsn, err := NewDsn("http://public@example.com/sentry/1")
	if err != nil {
		t.Fatal(err)
	}
	return dsn
}

func TestEnvelopeFromErrorBody(t *testing.T) {
	event := newTestEvent(eventType)
	sentAt := time.Unix(0, 0).UTC()

	body := json.RawMessage(`{"type":"event","fields":"omitted"}`)

	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"event","length":35}
{"type":"event","fields":"omitted"}
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}

func TestEnvelopeFromTransactionBody(t *testing.T) {
	event := newTestEvent(transactionType)
	sentAt := time.Unix(0, 0).UTC()

	body := json.RawMessage(`{"type":"transaction","fields":"omitted"}`)

	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"transaction","length":41}
{"type":"transaction","fields":"omitted"}
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}

func TestEnvelopeFromEventWithAttachments(t *testing.T) {
	event := newTestEvent(eventType)
	event.Attachments = []*Attachment{
		// Empty content-type and payload
		{
			Filename: "empty.txt",
		},
		// Non-empty content-type and payload
		{
			Filename:    "non-empty.txt",
			ContentType: "text/html",
			Payload:     []byte("<h1>Look, HTML</h1>"),
		},
	}
	sentAt := time.Unix(0, 0).UTC()

	body := json.RawMessage(`{"type":"event","fields":"omitted"}`)

	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"event","length":35}
{"type":"event","fields":"omitted"}
{"type":"attachment","length":0,"filename":"empty.txt"}

{"type":"attachment","length":19,"filename":"non-empty.txt","content_type":"text/html"}
<h1>Look, HTML</h1>
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}

func TestEnvelopeFromTransactionWithProfile(t *testing.T) {
	event := newTestEvent(transactionType)
	event.sdkMetaData.transactionProfile = &profileInfo{
		Trace: &profileTrace{
			Frames: []*Frame{
				{
					Function: "func",
					Module:   "module",
					Filename: "file.go",
					Lineno:   42,
					Colno:    24,
				},
			},
			Samples: []profileSample{
				{
					ElapsedSinceStartNS: 10,
					StackID:             2,
					ThreadID:            3,
				},
			},
			Stacks: []profileStack{{0}},
			ThreadMetadata: map[uint64]*profileThreadMetadata{
				1: {Name: "GO 1"},
			},
		},
		Transaction: profileTransaction{
			ActiveThreadID: 1,
			DurationNS:     2,
			ID:             "3",
			Name:           "tx-name",
			TraceID:        "trace-id",
		},
	}
	sentAt := time.Unix(0, 0).UTC()

	body := json.RawMessage(`{"type":"transaction","fields":"omitted"}`)

	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"transaction","length":41}
{"type":"transaction","fields":"omitted"}
{"type":"profile","length":618}
{"device":{"architecture":"","classification":"","locale":"","manufacturer":"","model":""},"event_id":"","os":{"build_number":"","name":"","version":""},"platform":"","release":"","dist":"","runtime":{"name":"","version":""},"timestamp":"0001-01-01T00:00:00Z","profile":{"frames":[{"function":"func","module":"module","filename":"file.go","lineno":42,"colno":24,"in_app":false}],"samples":[{"elapsed_since_start_ns":10,"stack_id":2,"thread_id":3}],"stacks":[[0]],"thread_metadata":{"1":{"name":"GO 1"}}},"transaction":{"active_thread_id":1,"duration_ns":2,"id":"3","name":"tx-name","trace_id":"trace-id"},"version":""}
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}

func TestEnvelopeFromCheckInEvent(t *testing.T) {
	event := newTestEvent(checkInType)
	event.CheckIn = &CheckIn{
		MonitorSlug: "test-slug",
		ID:          "123c5be4d31e48959103a1f878a1efcb",
		Status:      CheckInStatusOK,
	}
	event.MonitorConfig = &MonitorConfig{
		Schedule:      IntervalSchedule(1, MonitorScheduleUnitHour),
		CheckInMargin: 10,
		MaxRuntime:    5000,
		Timezone:      "Asia/Singapore",
	}
	sentAt := time.Unix(0, 0).UTC()

	body := getRequestBodyFromEvent(event)
	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"check_in","length":232}
{"check_in_id":"123c5be4d31e48959103a1f878a1efcb","monitor_slug":"test-slug","status":"ok","monitor_config":{"schedule":{"type":"interval","value":1,"unit":"hour"},"checkin_margin":10,"max_runtime":5000,"timezone":"Asia/Singapore"}}
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}

func TestGetRequestFromEvent(t *testing.T) {
	testCases := []struct {
		testName string
		// input
		event *Event
		// output
		apiURL string
	}{
		{
			testName: "Sample Event",
			event:    newTestEvent(eventType),
			apiURL:   "https://host/path/api/42/envelope/",
		},
		{
			testName: "Transaction",
			event:    newTestEvent(transactionType),
			apiURL:   "https://host/path/api/42/envelope/",
		},
	}

	for _, test := range testCases {
		test := test
		dsn, err := NewDsn("https://key@host/path/42")
		if err != nil {
			t.Fatal(err)
		}

		t.Run(test.testName, func(t *testing.T) {
			req, err := getRequestFromEvent(context.TODO(), test.event, dsn)
			if err != nil {
				t.Fatal(err)
			}

			if req.Method != http.MethodPost {
				t.Errorf("Request %v using http method: %s, supposed to use method: %s", req, req.Method, http.MethodPost)
			}

			if req.URL.String() != test.apiURL {
				t.Errorf("Incorrect API URL. want: %s, got: %s", test.apiURL, req.URL.String())
			}

			userAgent := fmt.Sprintf("%s/%s", test.event.Sdk.Name, test.event.Sdk.Version)
			if ua := req.UserAgent(); ua != userAgent {
				t.Errorf("got User-Agent = %q, want %q", ua, userAgent)
			}

			contentType := "application/x-sentry-envelope"
			if ct := req.Header.Get("Content-Type"); ct != contentType {
				t.Errorf("got Content-Type = %q, want %q", ct, contentType)
			}

			xSentryAuth := "Sentry sentry_version=7, sentry_client=sentry.go/0.0.1, sentry_key=key"
			if auth := req.Header.Get("X-Sentry-Auth"); !strings.HasPrefix(auth, xSentryAuth) {
				t.Errorf("got X-Sentry-Auth = %q, want %q", auth, xSentryAuth)
			}
		})
	}
}

// A testHTTPServer counts events sent to it. It requires a call to Unblock
// before incrementing its internal counter and sending a response to the HTTP
// client. This allows for coordinating the execution flow when needed.
type testHTTPServer struct {
	*httptest.Server
	// eventCounter counts the number of events processed by the server.
	eventCounter *uint64
	// ch is used to block/unblock the server on demand.
	ch chan bool
}

func newTestHTTPServer(t *testing.T) *testHTTPServer {
	ch := make(chan bool)
	eventCounter := new(uint64)
	handler := func(w http.ResponseWriter, r *http.Request) {
		var event struct {
			EventID string `json:"event_id"`
		}
		dec := json.NewDecoder(r.Body)
		err := dec.Decode(&event)
		if err != nil {
			t.Fatal(err)
		}
		// Block until signal to continue.
		<-ch
		count := atomic.AddUint64(eventCounter, 1)
		t.Logf("[SERVER] {%.4s} event received (#%d)", event.EventID, count)
	}
	return &testHTTPServer{
		Server:       httptest.NewTLSServer(http.HandlerFunc(handler)),
		eventCounter: eventCounter,
		ch:           ch,
	}
}

func (ts *testHTTPServer) EventCount() uint64 {
	return atomic.LoadUint64(ts.eventCounter)
}

func (ts *testHTTPServer) Unblock() {
	ts.ch <- true
}

func TestHTTPTransport(t *testing.T) {
	server := newTestHTTPServer(t)
	defer server.Close()

	transport := NewHTTPTransport()
	transport.Configure(ClientOptions{
		Dsn:        fmt.Sprintf("https://test@%s/1", server.Listener.Addr()),
		HTTPClient: server.Client(),
	})

	// Helpers

	transportSendTestEvent := func(t *testing.T) (id string) {
		t.Helper()

		e := NewEvent()
		id = uuid()
		e.EventID = EventID(id)

		transport.SendEvent(e)

		t.Logf("[CLIENT] {%.4s} event sent", e.EventID)
		return id
	}

	transportMustFlush := func(t *testing.T, id string) {
		t.Helper()
		ok := transport.Flush(testutils.FlushTimeout())
		if !ok {
			t.Fatalf("[CLIENT] {%.4s} Flush() timed out", id)
		}
	}

	serverEventCountMustBe := func(t *testing.T, n uint64) {
		t.Helper()

		count := server.EventCount()
		if count != n {
			t.Fatalf("[SERVER] event count = %d, want %d", count, n)
		}
	}

	testSendSingleEvent := func(t *testing.T) {
		// Sending a single event should increase the server event count by
		// exactly one.

		initialCount := server.EventCount()
		id := transportSendTestEvent(t)

		// Server is blocked waiting for us, right now count must not have
		// changed yet.
		serverEventCountMustBe(t, initialCount)

		// After unblocking the server, Flush must guarantee that the server
		// event count increased by one.
		server.Unblock()
		transportMustFlush(t, id)
		serverEventCountMustBe(t, initialCount+1)
	}

	// Actual tests
	t.Run("SendSingleEvent", testSendSingleEvent)

	t.Run("FlushMultipleTimes", func(t *testing.T) {
		// Flushing multiple times should not increase the server event count.

		initialCount := server.EventCount()
		for i := 0; i < 10; i++ {
			transportMustFlush(t, fmt.Sprintf("loop%d", i))
		}
		serverEventCountMustBe(t, initialCount)
	})

	t.Run("ConcurrentSendAndFlush", func(t *testing.T) {
		// It should be safe to send events and flush concurrently.

		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			testSendSingleEvent(t)
		}()
		go func() {
			defer wg.Done()
			transportMustFlush(t, "from goroutine")
		}()
		wg.Wait()
	})
}

// httptraceRoundTripper implements http.RoundTripper by wrapping
// http.DefaultTransport and keeps track of whether TCP connections have been
// reused for every request.
//
// For simplicity, httptraceRoundTripper is not safe for concurrent use.
type httptraceRoundTripper struct {
	reusedConn []bool
}

func (rt *httptraceRoundTripper) RoundTrip(req *http.Request) (*http.Response, error) {
	trace := &httptrace.ClientTrace{
		GotConn: func(connInfo httptrace.GotConnInfo) {
			rt.reusedConn = append(rt.reusedConn, connInfo.Reused)
		},
	}
	req = req.WithContext(httptrace.WithClientTrace(req.Context(), trace))
	return http.DefaultTransport.RoundTrip(req)
}

func testKeepAlive(t *testing.T, tr Transport) {
	// event is a test event. It is empty because here we only care about
	// the reuse of TCP connections between client and server, not the
	// specific contents of the event itself.
	event := &Event{}

	// largeResponse controls whether the test server should simulate an
	// unexpectedly large response from Relay -- the SDK should not try to
	// consume arbitrarily large responses.
	largeResponse := false
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Simulates a response from Relay. The event_id is arbitrary,
		// it doesn't matter for this test.
		fmt.Fprintln(w, `{"id":"ec71d87189164e79ab1e61030c183af0"}`)
		if largeResponse {
			fmt.Fprintln(w, strings.Repeat(" ", maxDrainResponseBytes))
		}
	}))
	defer srv.Close()

	dsn := strings.Replace(srv.URL, "//", "//pubkey@", 1) + "/1"

	rt := &httptraceRoundTripper{}
	tr.Configure(ClientOptions{
		Dsn:           dsn,
		HTTPTransport: rt,
	})

	reqCount := 0
	checkLastConnReuse := func(reused bool) {
		t.Helper()
		reqCount++
		if !tr.Flush(testutils.FlushTimeout()) {
			t.Fatal("Flush timed out")
		}
		if len(rt.reusedConn) != reqCount {
			t.Fatalf("unexpected number of requests: got %d, want %d", len(rt.reusedConn), reqCount)
		}
		if rt.reusedConn[reqCount-1] != reused {
			if reused {
				t.Fatal("TCP connection not reused")
			}
			t.Fatal("unexpected TCP connection reuse")
		}
	}

	// First event creates a new TCP connection.
	tr.SendEvent(event)
	checkLastConnReuse(false)

	// Next events reuse the TCP connection.
	for i := 0; i < 10; i++ {
		tr.SendEvent(event)
		checkLastConnReuse(true)
	}

	// If server responses are too large, the SDK should close the
	// connection instead of consuming an arbitrarily large number of bytes.
	largeResponse = true

	// Next event, first one to get a large response, reuses the connection.
	tr.SendEvent(event)
	checkLastConnReuse(true)

	// All future events create a new TCP connection.
	for i := 0; i < 10; i++ {
		tr.SendEvent(event)
		checkLastConnReuse(false)
	}
}

func TestKeepAlive(t *testing.T) {
	t.Run("AsyncTransport", func(t *testing.T) {
		testKeepAlive(t, NewHTTPTransport())
	})
	t.Run("SyncTransport", func(t *testing.T) {
		testKeepAlive(t, NewHTTPSyncTransport())
	})
}

func TestRateLimiting(t *testing.T) {
	t.Run("AsyncTransport", func(t *testing.T) {
		testRateLimiting(t, NewHTTPTransport())
	})
	t.Run("SyncTransport", func(t *testing.T) {
		testRateLimiting(t, NewHTTPSyncTransport())
	})
}

func testRateLimiting(t *testing.T, tr Transport) {
	errorEvent := &Event{}
	transactionEvent := &Event{Type: transactionType}

	var errorEventCount, transactionEventCount uint64

	writeRateLimits := func(w http.ResponseWriter, s string) {
		w.Header().Add("Retry-After", "50")
		w.Header().Add("X-Sentry-Rate-Limits", s)
		w.WriteHeader(http.StatusTooManyRequests)
		fmt.Fprint(w, `{"id":"636205708f6846c8821e6576a9d05921"}`)
	}

	// Test server that simulates responses with rate limits.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, err := io.ReadAll(r.Body)
		if err != nil {
			panic(err)
		}
		if bytes.Contains(b, []byte("transaction")) {
			atomic.AddUint64(&transactionEventCount, 1)
			writeRateLimits(w, "20:transaction")
		} else {
			atomic.AddUint64(&errorEventCount, 1)
			writeRateLimits(w, "50:error")
		}
	}))
	defer srv.Close()

	dsn := strings.Replace(srv.URL, "//", "//pubkey@", 1) + "/1"

	tr.Configure(ClientOptions{
		Dsn: dsn,
	})

	// Send several errors and transactions concurrently.
	//
	// Because the server always returns a rate limit for the payload type
	// in the request, the expectation is that, for both errors and
	// transactions, the first event is sent successfully, and then all
	// others are discarded before hitting the server.
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		for i := 0; i < 10; i++ {
			tr.SendEvent(errorEvent)
		}
	}()
	go func() {
		defer wg.Done()
		for i := 0; i < 10; i++ {
			tr.SendEvent(transactionEvent)
		}
	}()
	wg.Wait()

	if !tr.Flush(testutils.FlushTimeout()) {
		t.Fatal("Flush timed out")
	}

	// Only one event of each kind should have hit the transport, all other
	// events discarded because of rate limiting.
	if n := atomic.LoadUint64(&errorEventCount); n != 1 {
		t.Errorf("got errorEvent = %d, want %d", n, 1)
	}
	if n := atomic.LoadUint64(&transactionEventCount); n != 1 {
		t.Errorf("got transactionEvent = %d, want %d", n, 1)
	}
}

func TestEnvelopeFromMetricBody(t *testing.T) {
	event := newTestEvent(metricType)
	event.Metrics = append(event.Metrics,
		NewCounterMetric("counter", Second(), map[string]string{"foo": "bar", "route": "GET /foo"}, 1597790835, 1.0),
		NewDistributionMetric("distribution", Second(), map[string]string{"$foo$": "%bar%"}, 1597790835, 1.0),
		NewGaugeMetric("gauge", Second(), map[string]string{"föö": "bär"}, 1597790835, 1.0),
		NewSetMetric[int]("set", Second(), map[string]string{"%{key}": "$value$"}, 1597790835, 1),
		NewCounterMetric("no_tags", Second(), nil, 1597790835, 1.0),
	)
	sentAt := time.Unix(0, 0).UTC()

	body := getRequestBodyFromEvent(event)

	b, err := envelopeFromBody(event, newTestDSN(t), sentAt, body)
	if err != nil {
		t.Fatal(err)
	}
	got := b.String()
	want := `{"event_id":"b81c5be4d31e48959103a1f878a1efcb","sent_at":"1970-01-01T00:00:00Z","dsn":"http://public@example.com/sentry/1","sdk":{"name":"sentry.go","version":"0.0.1"}}
{"type":"statsd","length":218}
counter@second:1|c|#foo:bar,route:GET /foo|T1597790835
distribution@second:1|d|#_foo_:bar|T1597790835
gauge@second:1:1:1:1:1|g|#f_:br|T1597790835
set@second:1|s|#_key_:$value$|T1597790835
no_tags@second:1|c|T1597790835
`
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Envelope mismatch (-want +got):\n%s", diff)
	}
}
