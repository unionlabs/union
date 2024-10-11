package sentryfasthttp_test

import (
	"errors"
	"fmt"
	"net"
	"net/http"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/getsentry/sentry-go"
	sentryfasthttp "github.com/getsentry/sentry-go/fasthttp"
	"github.com/getsentry/sentry-go/internal/testutils"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/valyala/fasthttp"
	"github.com/valyala/fasthttp/fasthttputil"
)

func TestIntegration(t *testing.T) {
	largePayload := strings.Repeat("Large", 3*1024) // 15 KB

	exception := errors.New("unknown error")

	tests := []struct {
		Path            string
		Method          string
		Body            string
		WantStatus      int
		Handler         fasthttp.RequestHandler
		WantEvent       *sentry.Event
		WantTransaction *sentry.Event
	}{
		{
			Path: "/panic",
			Handler: func(*fasthttp.RequestCtx) {
				panic("test")
			},
			WantStatus: 200,
			WantEvent: &sentry.Event{
				Level:   sentry.LevelFatal,
				Message: "test",
				Request: &sentry.Request{
					URL:    "http://example.com/panic",
					Method: http.MethodGet,
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "GET /panic",
				Request: &sentry.Request{
					URL:    "http://example.com/panic",
					Method: http.MethodGet,
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodGet, "http.response.status_code": http.StatusOK},
			},
		},
		{
			Path:       "/post",
			Method:     http.MethodPost,
			WantStatus: 200,
			Body:       "payload",
			Handler: func(ctx *fasthttp.RequestCtx) {
				hub := sentryfasthttp.GetHubFromContext(ctx)
				hub.CaptureMessage("post: " + string(ctx.Request.Body()))
			},
			WantEvent: &sentry.Event{
				Level:   sentry.LevelInfo,
				Message: "post: payload",
				Request: &sentry.Request{
					URL:    "http://example.com/post",
					Method: http.MethodPost,
					Data:   "payload",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "POST /post",
				Request: &sentry.Request{
					URL:    "http://example.com/post",
					Method: http.MethodPost,
					Data:   "payload",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodPost, "http.response.status_code": http.StatusOK},
			},
		},
		{
			Path: "/get",
			Handler: func(ctx *fasthttp.RequestCtx) {
				hub := sentryfasthttp.GetHubFromContext(ctx)
				hub.CaptureMessage(http.MethodGet)
			},
			WantStatus: 200,
			WantEvent: &sentry.Event{
				Level:   sentry.LevelInfo,
				Message: http.MethodGet,
				Request: &sentry.Request{
					URL:    "http://example.com/get",
					Method: http.MethodGet,
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "GET /get",
				Request: &sentry.Request{
					URL:    "http://example.com/get",
					Method: http.MethodGet,
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodGet, "http.response.status_code": http.StatusOK},
			},
		},
		{
			Path:       "/post/large",
			Method:     http.MethodPost,
			Body:       largePayload,
			WantStatus: 200,
			Handler: func(ctx *fasthttp.RequestCtx) {
				hub := sentryfasthttp.GetHubFromContext(ctx)
				hub.CaptureMessage(fmt.Sprintf("post: %d KB", len(ctx.Request.Body())/1024))
			},
			WantEvent: &sentry.Event{
				Level:   sentry.LevelInfo,
				Message: "post: 15 KB",
				Request: &sentry.Request{
					URL:    "http://example.com/post/large",
					Method: http.MethodPost,
					// Actual request body omitted because too large.
					Data: "",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "POST /post/large",
				Request: &sentry.Request{
					URL:    "http://example.com/post/large",
					Method: http.MethodPost,
					Data:   "",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodPost, "http.response.status_code": http.StatusOK},
			},
		},
		{
			Path:       "/post/body-ignored",
			Method:     http.MethodPost,
			Body:       "client sends, fasthttp always reads, SDK reports",
			WantStatus: 200,
			Handler: func(ctx *fasthttp.RequestCtx) {
				hub := sentryfasthttp.GetHubFromContext(ctx)
				hub.CaptureMessage("body ignored")
			},
			WantEvent: &sentry.Event{
				Level:   sentry.LevelInfo,
				Message: "body ignored",
				Request: &sentry.Request{
					URL:    "http://example.com/post/body-ignored",
					Method: http.MethodPost,
					// Actual request body included because fasthttp always
					// reads full request body.
					Data: "client sends, fasthttp always reads, SDK reports",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "POST /post/body-ignored",
				Request: &sentry.Request{
					URL:    "http://example.com/post/body-ignored",
					Method: http.MethodPost,
					Data:   "client sends, fasthttp always reads, SDK reports",
					Headers: map[string]string{
						"Host":       "example.com",
						"User-Agent": "fasthttp",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodPost, "http.response.status_code": http.StatusOK},
			},
		},
		{
			Path:   "/post/error-handler",
			Method: "POST",
			Handler: func(ctx *fasthttp.RequestCtx) {
				hub := sentryfasthttp.GetHubFromContext(ctx)
				hub.CaptureException(exception)
			},
			WantStatus: 200,
			WantEvent: &sentry.Event{
				Level: sentry.LevelError,
				Exception: []sentry.Exception{
					{
						Value: exception.Error(),
						Type:  reflect.TypeOf(exception).String(),
					},
				},
				Request: &sentry.Request{
					URL:    "http://example.com/post/error-handler",
					Method: "POST",
					Headers: map[string]string{
						"Content-Length": "0",
						"Host":           "example.com",
						"User-Agent":     "fasthttp",
					},
				},
			},
			WantTransaction: &sentry.Event{
				Level:       sentry.LevelInfo,
				Type:        "transaction",
				Transaction: "POST /post/error-handler",
				Request: &sentry.Request{
					URL:    "http://example.com/post/error-handler",
					Method: http.MethodPost,
					Headers: map[string]string{
						"Host":           "example.com",
						"User-Agent":     "fasthttp",
						"Content-Length": "0",
					},
				},
				TransactionInfo: &sentry.TransactionInfo{Source: "route"},
				Extra:           map[string]any{"http.request.method": http.MethodPost, "http.response.status_code": http.StatusOK},
			},
		},
	}

	eventsCh := make(chan *sentry.Event, len(tests))
	transactionsCh := make(chan *sentry.Event, len(tests))
	err := sentry.Init(sentry.ClientOptions{
		EnableTracing:    true,
		TracesSampleRate: 1.0,
		BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			eventsCh <- event
			return event
		},
		BeforeSendTransaction: func(tx *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			transactionsCh <- tx
			return tx
		},
	})
	if err != nil {
		t.Fatal(err)
	}

	sentryHandler := sentryfasthttp.New(sentryfasthttp.Options{})
	ln := fasthttputil.NewInmemoryListener()
	handler := func(ctx *fasthttp.RequestCtx) {
		for _, tt := range tests {
			if string(ctx.Path()) == tt.Path {
				tt.Handler(ctx)
				return
			}
		}
		t.Errorf("Unhandled request: %#v", ctx)
	}
	done := make(chan struct{})
	go func() {
		if err := fasthttp.Serve(ln, sentryHandler.Handle(handler)); err != nil {
			t.Errorf("error in Serve: %s", err)
		}
		close(done)
	}()

	c := &fasthttp.Client{
		Dial: func(addr string) (net.Conn, error) {
			return ln.Dial()
		},
		ReadTimeout:  time.Second,
		WriteTimeout: time.Second,
	}

	var wantEvents []*sentry.Event
	var wantTransactions []*sentry.Event
	var wantCodes []sentry.SpanStatus
	for _, tt := range tests {
		wantEvents = append(wantEvents, tt.WantEvent)
		wantTransactions = append(wantTransactions, tt.WantTransaction)
		wantCodes = append(wantCodes, sentry.HTTPtoSpanStatus(tt.WantStatus))
		req, res := fasthttp.AcquireRequest(), fasthttp.AcquireResponse()
		req.SetHost("example.com")
		req.URI().SetPath(tt.Path)
		req.Header.SetMethod(tt.Method)
		req.SetBodyString(tt.Body)
		if err := c.Do(req, res); err != nil {
			t.Fatalf("Request %q failed: %s", tt.Path, err)
		}
		if res.StatusCode() != http.StatusOK {
			t.Errorf("Status code = %d", res.StatusCode())
		}
	}

	if ok := sentry.Flush(testutils.FlushTimeout()); !ok {
		t.Fatal("sentry.Flush timed out")
	}

	close(eventsCh)
	var gotEvents []*sentry.Event
	for e := range eventsCh {
		gotEvents = append(gotEvents, e)
	}

	opts := cmp.Options{
		cmpopts.IgnoreFields(
			sentry.Event{},
			"Contexts", "EventID", "Extra", "Platform", "Modules",
			"Release", "Sdk", "ServerName", "Tags", "Timestamp",
			"sdkMetaData",
		),
		cmpopts.IgnoreFields(
			sentry.Exception{},
			"Stacktrace",
		),
		cmpopts.IgnoreMapEntries(func(k string, v string) bool {
			// fasthttp changed Content-Length behavior in
			// https://github.com/valyala/fasthttp/commit/097fa05a697fc638624a14ab294f1336da9c29b0.
			// fasthttp changed Content-Type behavior in
			// https://github.com/valyala/fasthttp/commit/ffa0cabed8199819e372ebd2c739998914150ff2.
			// Since the specific values of those headers are not
			// important from the perspective of sentry-go, we
			// ignore them.
			return k == "Content-Length" || k == "Content-Type"
		}),
	}
	if diff := cmp.Diff(wantEvents, gotEvents, opts); diff != "" {
		t.Fatalf("Events mismatch (-want +gotEvents):\n%s", diff)
	}

	close(transactionsCh)
	var gotTransactions []*sentry.Event
	var statusCodes []sentry.SpanStatus

	for e := range transactionsCh {
		gotTransactions = append(gotTransactions, e)
		statusCodes = append(statusCodes, e.Contexts["trace"]["status"].(sentry.SpanStatus))
	}
	optstrans := cmp.Options{
		cmpopts.IgnoreFields(
			sentry.Event{},
			"Contexts", "EventID", "Platform", "Modules",
			"Release", "Sdk", "ServerName", "Timestamp",
			"sdkMetaData", "StartTime", "Spans",
		),
		cmpopts.IgnoreFields(
			sentry.Request{},
			"Env",
		),
		cmpopts.IgnoreFields(
			sentry.Exception{},
			"Stacktrace",
		),
		cmpopts.IgnoreMapEntries(func(k string, v string) bool {
			// fasthttp changed Content-Length behavior in
			// https://github.com/valyala/fasthttp/commit/097fa05a697fc638624a14ab294f1336da9c29b0.
			// fasthttp changed Content-Type behavior in
			// https://github.com/valyala/fasthttp/commit/ffa0cabed8199819e372ebd2c739998914150ff2.
			// Since the specific values of those headers are not
			// important from the perspective of sentry-go, we
			// ignore them.
			return k == "Content-Length" || k == "Content-Type"
		}),
	}
	if diff := cmp.Diff(wantTransactions, gotTransactions, optstrans); diff != "" {
		t.Fatalf("Transactions mismatch (-want +gotEvents):\n%s", diff)
	}

	if diff := cmp.Diff(wantCodes, statusCodes, cmp.Options{}); diff != "" {
		t.Fatalf("Transaction status codes mismatch (-want +got):\n%s", diff)
	}

	ln.Close()
	<-done
}

func TestGetTransactionFromContext(t *testing.T) {
	err := sentry.Init(sentry.ClientOptions{
		EnableTracing:    true,
		TracesSampleRate: 1.0,
	})
	if err != nil {
		t.Fatal(err)
	}

	tests := map[string]struct {
		useSentry bool
	}{
		"With Transaction":    {useSentry: true},
		"Without Transaction": {useSentry: false},
	}

	for name, tc := range tests {
		t.Run(name, func(t *testing.T) {
			ln := fasthttputil.NewInmemoryListener()
			defer ln.Close()

			handler := func(ctx *fasthttp.RequestCtx) {
				span := sentryfasthttp.GetSpanFromContext(ctx)
				if tc.useSentry && span == nil {
					t.Error("expecting span not to be nil")
				}
				if !tc.useSentry && span != nil {
					t.Error("expecting span to be nil")
				}
				ctx.SetStatusCode(200)
			}

			var finalHandler fasthttp.RequestHandler
			if tc.useSentry {
				sentryHandler := sentryfasthttp.New(sentryfasthttp.Options{})
				finalHandler = sentryHandler.Handle(handler)
			} else {
				finalHandler = handler
			}

			done := make(chan struct{})
			go func() {
				if err := fasthttp.Serve(ln, finalHandler); err != nil {
					t.Errorf("error in Serve: %s", err)
				}
				close(done)
			}()

			c := &fasthttp.Client{
				Dial: func(addr string) (net.Conn, error) {
					return ln.Dial()
				},
				ReadTimeout:  time.Second,
				WriteTimeout: time.Second,
			}

			req, res := fasthttp.AcquireRequest(), fasthttp.AcquireResponse()
			req.SetHost("example.com")
			req.URI().SetPath("/")
			req.Header.SetMethod(http.MethodGet)

			if err := c.Do(req, res); err != nil {
				t.Fatalf("Request failed: %s", err)
			}
			if res.StatusCode() != http.StatusOK {
				t.Errorf("Status code = %d", res.StatusCode())
			}

			// Cleanup
			fasthttp.ReleaseRequest(req)
			fasthttp.ReleaseResponse(res)

			ln.Close()
			<-done
		})
	}
}
