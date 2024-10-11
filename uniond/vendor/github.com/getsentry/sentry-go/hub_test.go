package sentry

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
)

const testDsn = "http://whatever@example.com/1337"

func setupHubTest() (*Hub, *Client, *Scope) {
	client, _ := NewClient(ClientOptions{Dsn: testDsn, Transport: &TransportMock{}})
	scope := NewScope()
	hub := NewHub(client, scope)
	return hub, client, scope
}

func TestNewHubPushesLayerOnTopOfStack(t *testing.T) {
	hub, _, _ := setupHubTest()
	assertEqual(t, len(*hub.stack), 1)
}

func TestNewHubLayerStoresClientAndScope(t *testing.T) {
	hub, client, scope := setupHubTest()
	assertEqual(t, &layer{client: client, scope: scope}, (*hub.stack)[0])
}

func TestCloneHubInheritsClientAndScope(t *testing.T) {
	hub, client, scope := setupHubTest()
	clone := hub.Clone()

	if hub == clone {
		t.Error("Cloned hub should be a new instance")
	}

	if clone.Client() != client {
		t.Error("Client should be inherited")
	}

	if clone.Scope() == scope {
		t.Error("Scope should be cloned, not reused")
	}

	assertEqual(t, clone.Scope(), scope)
}

func TestPushScopeAddsScopeOnTopOfStack(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.PushScope()
	assertEqual(t, len(*hub.stack), 2)
}

func TestPushScopeInheritsScopeData(t *testing.T) {
	hub, _, scope := setupHubTest()
	scope.SetExtra("foo", "bar")
	hub.PushScope()
	scope.SetExtra("baz", "qux")

	if (*hub.stack)[0].scope == (*hub.stack)[1].scope {
		t.Error("Scope shouldnt point to the same struct")
	}
	assertEqual(t, map[string]interface{}{"foo": "bar", "baz": "qux"}, (*hub.stack)[0].scope.extra)
	assertEqual(t, map[string]interface{}{"foo": "bar"}, (*hub.stack)[1].scope.extra)
}

func TestPushScopeInheritsClient(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.PushScope()

	if (*hub.stack)[0].client != (*hub.stack)[1].client {
		t.Error("Client should be inherited")
	}
}

func TestPopScopeRemovesLayerFromTheStack(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.PushScope()
	hub.PushScope()
	hub.PopScope()

	assertEqual(t, len(*hub.stack), 2)
}

func TestPopScopeCannotLeaveStackEmpty(t *testing.T) {
	hub, _, _ := setupHubTest()
	assertEqual(t, len(*hub.stack), 1)
	hub.PopScope()
	assertEqual(t, len(*hub.stack), 1)
}

func TestBindClient(t *testing.T) {
	hub, client, _ := setupHubTest()
	hub.PushScope()
	newClient, _ := NewClient(ClientOptions{Dsn: testDsn, Transport: &TransportMock{}})
	hub.BindClient(newClient)

	if (*hub.stack)[0].client == (*hub.stack)[1].client {
		t.Error("Two stack layers should have different clients bound")
	}
	if (*hub.stack)[0].client != client {
		t.Error("Stack's parent layer should have old client bound")
	}
	if (*hub.stack)[1].client != newClient {
		t.Error("Stack's top layer should have new client bound")
	}
}

func TestWithScopeCreatesIsolatedScope(t *testing.T) {
	hub, _, _ := setupHubTest()

	hub.WithScope(func(scope *Scope) {
		assertEqual(t, len(*hub.stack), 2)
	})

	assertEqual(t, len(*hub.stack), 1)
}

func TestWithScopeBindClient(t *testing.T) {
	hub, client, _ := setupHubTest()

	hub.WithScope(func(scope *Scope) {
		newClient, _ := NewClient(ClientOptions{Dsn: testDsn, Transport: &TransportMock{}})
		hub.BindClient(newClient)
		if hub.stackTop().client != newClient {
			t.Error("should use newly bound client")
		}
	})

	if hub.stackTop().client != client {
		t.Error("should use old client")
	}
}

func TestWithScopeDirectChanges(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.Scope().SetExtra("extra", "foo")

	hub.WithScope(func(scope *Scope) {
		scope.SetExtra("extra", "bar")
		assertEqual(t, map[string]interface{}{"extra": "bar"}, hub.stackTop().scope.extra)
	})

	assertEqual(t, map[string]interface{}{"extra": "foo"}, hub.stackTop().scope.extra)
}

func TestWithScopeChangesThroughConfigureScope(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.Scope().SetExtra("extra", "foo")

	hub.WithScope(func(scope *Scope) {
		hub.ConfigureScope(func(scope *Scope) {
			scope.SetExtra("extra", "bar")
		})
		assertEqual(t, map[string]interface{}{"extra": "bar"}, hub.stackTop().scope.extra)
	})

	assertEqual(t, map[string]interface{}{"extra": "foo"}, hub.stackTop().scope.extra)
}

func TestConfigureScope(t *testing.T) {
	hub, _, _ := setupHubTest()
	hub.Scope().SetExtra("extra", "foo")

	hub.ConfigureScope(func(scope *Scope) {
		scope.SetExtra("extra", "bar")
		assertEqual(t, map[string]interface{}{"extra": "bar"}, hub.stackTop().scope.extra)
	})

	assertEqual(t, map[string]interface{}{"extra": "bar"}, hub.stackTop().scope.extra)
}

func TestLastEventID(t *testing.T) {
	uuid := EventID(uuid())
	hub := &Hub{lastEventID: uuid}
	assertEqual(t, uuid, hub.LastEventID())
}

func TestLastEventIDUpdatesAfterCaptures(t *testing.T) {
	hub, _, _ := setupHubTest()

	messageID := hub.CaptureMessage("wat")
	assertEqual(t, *messageID, hub.LastEventID())

	errorID := hub.CaptureException(fmt.Errorf("wat"))
	assertEqual(t, *errorID, hub.LastEventID())

	eventID := hub.CaptureEvent(&Event{Message: "wat"})
	assertEqual(t, *eventID, hub.LastEventID())
}

func TestLastEventIDNotChangedForTransactions(t *testing.T) {
	hub, _, _ := setupHubTest()

	errorID := hub.CaptureException(fmt.Errorf("wat"))
	assertEqual(t, *errorID, hub.LastEventID())

	hub.CaptureEvent(&Event{Type: transactionType})
	assertEqual(t, *errorID, hub.LastEventID())
}

func TestLastEventIDDoesNotReset(t *testing.T) {
	hub, client, _ := setupHubTest()

	id1 := hub.CaptureException(fmt.Errorf("error 1"))
	assertEqual(t, hub.LastEventID(), *id1)

	client.AddEventProcessor(func(event *Event, hint *EventHint) *Event {
		// drop all events
		return nil
	})

	id2 := hub.CaptureException(fmt.Errorf("error 2"))
	assertEqual(t, id2, (*EventID)(nil))    // event must have been dropped
	assertEqual(t, hub.LastEventID(), *id1) // last event ID must not have changed
}

func TestAddBreadcrumbRespectMaxBreadcrumbsOption(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.MaxBreadcrumbs = 2

	breadcrumb := &Breadcrumb{Message: "Breadcrumb"}

	hub.AddBreadcrumb(breadcrumb, nil)
	hub.AddBreadcrumb(breadcrumb, nil)
	hub.AddBreadcrumb(breadcrumb, nil)

	assertEqual(t, len(scope.breadcrumbs), 2)
}

func TestAddBreadcrumbSkipAllBreadcrumbsIfMaxBreadcrumbsIsLessThanZero(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.MaxBreadcrumbs = -1

	breadcrumb := &Breadcrumb{Message: "Breadcrumb"}

	hub.AddBreadcrumb(breadcrumb, nil)
	hub.AddBreadcrumb(breadcrumb, nil)
	hub.AddBreadcrumb(breadcrumb, nil)

	assertEqual(t, len(scope.breadcrumbs), 0)
}

func TestAddBreadcrumbShouldNeverExceedMaxBreadcrumbsConst(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.MaxBreadcrumbs = 1000

	breadcrumb := &Breadcrumb{Message: "Breadcrumb"}

	for i := 0; i < 111; i++ {
		hub.AddBreadcrumb(breadcrumb, nil)
	}

	assertEqual(t, len(scope.breadcrumbs), 100)
}

func TestAddBreadcrumbShouldWorkWithoutClient(t *testing.T) {
	scope := NewScope()
	hub := NewHub(nil, scope)

	breadcrumb := &Breadcrumb{Message: "Breadcrumb"}
	for i := 0; i < 111; i++ {
		hub.AddBreadcrumb(breadcrumb, nil)
	}

	assertEqual(t, len(scope.breadcrumbs), 100)
}

func TestAddBreadcrumbCallsBeforeBreadcrumbCallback(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.BeforeBreadcrumb = func(breadcrumb *Breadcrumb, hint *BreadcrumbHint) *Breadcrumb {
		breadcrumb.Message += "_wat"
		return breadcrumb
	}

	hub.AddBreadcrumb(&Breadcrumb{Message: "Breadcrumb"}, nil)

	assertEqual(t, len(scope.breadcrumbs), 1)
	assertEqual(t, "Breadcrumb_wat", scope.breadcrumbs[0].Message)
}

func TestBeforeBreadcrumbCallbackCanDropABreadcrumb(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.BeforeBreadcrumb = func(breadcrumb *Breadcrumb, hint *BreadcrumbHint) *Breadcrumb {
		return nil
	}

	hub.AddBreadcrumb(&Breadcrumb{Message: "Breadcrumb"}, nil)
	hub.AddBreadcrumb(&Breadcrumb{Message: "Breadcrumb"}, nil)

	assertEqual(t, len(scope.breadcrumbs), 0)
}

func TestBeforeBreadcrumbGetAccessToEventHint(t *testing.T) {
	hub, client, scope := setupHubTest()
	client.options.BeforeBreadcrumb = func(breadcrumb *Breadcrumb, hint *BreadcrumbHint) *Breadcrumb {
		if val, ok := (*hint)["foo"]; ok {
			if val, ok := val.(string); ok {
				breadcrumb.Message += val
			}
		}

		return breadcrumb
	}

	hub.AddBreadcrumb(&Breadcrumb{Message: "Breadcrumb"}, &BreadcrumbHint{"foo": "_oh"})

	assertEqual(t, len(scope.breadcrumbs), 1)
	assertEqual(t, "Breadcrumb_oh", scope.breadcrumbs[0].Message)
}

func TestHasHubOnContextReturnsTrueIfHubIsThere(t *testing.T) {
	hub, _, _ := setupHubTest()
	ctx := context.Background()
	ctx = SetHubOnContext(ctx, hub)
	assertEqual(t, true, HasHubOnContext(ctx))
}

func TestHasHubOnContextReturnsFalseIfHubIsNotThere(t *testing.T) {
	ctx := context.Background()
	assertEqual(t, false, HasHubOnContext(ctx))
}

func TestGetTraceparent(t *testing.T) {
	tests := map[string]struct {
		hub      *Hub
		expected string
	}{
		"With span": {
			hub: func() *Hub {
				h, _, s := setupHubTest()
				s.span = &Span{
					TraceID: TraceIDFromHex("d49d9bf66f13450b81f65bc51cf49c03"),
					SpanID:  SpanIDFromHex("a9f442f9330b4e09"),
					Sampled: SampledTrue,
				}
				return h
			}(),
			expected: "d49d9bf66f13450b81f65bc51cf49c03-a9f442f9330b4e09-1",
		},
		"Without span": {
			hub: func() *Hub {
				h, _, s := setupHubTest()
				s.propagationContext.TraceID = TraceIDFromHex("d49d9bf66f13450b81f65bc51cf49c03")
				s.propagationContext.SpanID = SpanIDFromHex("a9f442f9330b4e09")
				return h
			}(),
			expected: "d49d9bf66f13450b81f65bc51cf49c03-a9f442f9330b4e09",
		},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			result := tt.hub.GetTraceparent()
			assertEqual(t, result, tt.expected)
		})
	}
}

func TestGetBaggage(t *testing.T) {
	tests := map[string]struct {
		hub      *Hub
		expected string
	}{
		"With span": {
			hub: func() *Hub {
				h, _, s := setupHubTest()
				s.span = &Span{
					dynamicSamplingContext: DynamicSamplingContext{
						Entries: map[string]string{"sample_rate": "1", "release": "1.0.0", "environment": "production"},
					},
					recorder: &spanRecorder{},
					ctx:      context.Background(),
					Sampled:  SampledTrue,
				}

				s.span.spanRecorder().record(s.span)

				return h
			}(),
			expected: "sentry-environment=production,sentry-release=1.0.0,sentry-sample_rate=1",
		},
		"Without span": {
			hub: func() *Hub {
				h, _, s := setupHubTest()
				s.propagationContext.DynamicSamplingContext = DynamicSamplingContext{
					Entries: map[string]string{"release": "1.0.0", "environment": "production"},
				}
				return h
			}(),
			expected: "sentry-environment=production,sentry-release=1.0.0",
		},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			result := tt.hub.GetBaggage()
			res := strings.Split(result, ",")
			sortSlice(res)
			assertEqual(t, strings.Join(res, ","), tt.expected)
		})
	}
}

func TestGetHubFromContext(t *testing.T) {
	hub, _, _ := setupHubTest()
	ctx := context.Background()
	ctx = SetHubOnContext(ctx, hub)
	hubFromContext := GetHubFromContext(ctx)
	assertEqual(t, hub, hubFromContext)
}

func TestGetHubFromContextReturnsNilIfHubIsNotThere(t *testing.T) {
	ctx := context.Background()
	hub := GetHubFromContext(ctx)
	if hub != nil {
		t.Error("hub shouldnt be available on empty context")
	}
}

func TestSetHubOnContextReturnsNewContext(t *testing.T) {
	hub, _, _ := setupHubTest()
	ctx := context.Background()
	ctxWithHub := SetHubOnContext(ctx, hub)
	if ctx == ctxWithHub {
		t.Error("contexts should be different")
	}
}

func TestConcurrentHubClone(t *testing.T) {
	const goroutineCount = 3

	hub, client, _ := setupHubTest()
	transport := &TransportMock{}
	client.Transport = transport

	var wg sync.WaitGroup
	wg.Add(goroutineCount)
	for i := 1; i <= goroutineCount; i++ {
		// Mutate hub in the main goroutine.
		hub.PushScope()
		hub.PopScope()
		hub.BindClient(client)
		// Clone scope in a new Goroutine as documented in
		// https://docs.sentry.io/platforms/go/goroutines/.
		go func(i int) {
			defer wg.Done()
			localHub := hub.Clone()
			localHub.ConfigureScope(func(scope *Scope) {
				scope.SetTag("secretTag", fmt.Sprintf("go#%d", i))
			})
			localHub.CaptureMessage(fmt.Sprintf("Hello from goroutine! #%d", i))
		}(i)
	}
	wg.Wait()

	type TestEvent struct {
		Message string
		Tags    map[string]string
	}

	want := []TestEvent{
		{
			Message: "Hello from goroutine! #1",
			Tags:    map[string]string{"secretTag": "go#1"},
		},
		{
			Message: "Hello from goroutine! #2",
			Tags:    map[string]string{"secretTag": "go#2"},
		},
		{
			Message: "Hello from goroutine! #3",
			Tags:    map[string]string{"secretTag": "go#3"},
		},
	}

	var got []TestEvent
	for _, event := range transport.Events() {
		got = append(got, TestEvent{
			Message: event.Message,
			Tags:    event.Tags,
		})
	}

	if diff := cmp.Diff(want, got, cmpopts.SortSlices(func(x, y TestEvent) bool {
		return x.Message < y.Message
	})); diff != "" {
		t.Errorf("Events mismatch (-want +got):\n%s", diff)
	}
}
