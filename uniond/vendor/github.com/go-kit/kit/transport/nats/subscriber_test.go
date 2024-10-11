package nats_test

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/nats-io/nats-server/v2/server"
	"github.com/nats-io/nats.go"

	"github.com/go-kit/kit/endpoint"
	natstransport "github.com/go-kit/kit/transport/nats"
)

type TestResponse struct {
	String string `json:"str"`
	Error  string `json:"err"`
}

func newNATSConn(t *testing.T) (*server.Server, *nats.Conn) {
	s, err := server.NewServer(&server.Options{
		Host: "localhost",
		Port: 0,
	})
	if err != nil {
		t.Fatal(err)
	}

	go s.Start()

	for i := 0; i < 5 && !s.Running(); i++ {
		t.Logf("Running %v", s.Running())
		time.Sleep(time.Second)
	}
	if !s.Running() {
		s.Shutdown()
		s.WaitForShutdown()
		t.Fatal("not yet running")
	}

	if ok := s.ReadyForConnections(5 * time.Second); !ok {
		t.Fatal("not ready for connections")
	}

	c, err := nats.Connect("nats://"+s.Addr().String(), nats.Name(t.Name()))
	if err != nil {
		t.Fatalf("failed to connect to NATS server: %s", err)
	}

	return s, c
}

func TestSubscriberBadDecode(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) { return struct{}{}, nil },
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, errors.New("dang") },
		func(context.Context, string, *nats.Conn, interface{}) error { return nil },
	)

	resp := testRequest(t, c, handler)

	if want, have := "dang", resp.Error; want != have {
		t.Errorf("want %s, have %s", want, have)
	}

}

func TestSubscriberBadEndpoint(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) { return struct{}{}, errors.New("dang") },
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		func(context.Context, string, *nats.Conn, interface{}) error { return nil },
	)

	resp := testRequest(t, c, handler)

	if want, have := "dang", resp.Error; want != have {
		t.Errorf("want %s, have %s", want, have)
	}
}

func TestSubscriberBadEncode(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) { return struct{}{}, nil },
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		func(context.Context, string, *nats.Conn, interface{}) error { return errors.New("dang") },
	)

	resp := testRequest(t, c, handler)

	if want, have := "dang", resp.Error; want != have {
		t.Errorf("want %s, have %s", want, have)
	}
}

func TestSubscriberErrorEncoder(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	errTeapot := errors.New("teapot")
	code := func(err error) error {
		if errors.Is(err, errTeapot) {
			return err
		}
		return errors.New("dang")
	}
	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) { return struct{}{}, errTeapot },
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		func(context.Context, string, *nats.Conn, interface{}) error { return nil },
		natstransport.SubscriberErrorEncoder(func(_ context.Context, err error, reply string, nc *nats.Conn) {
			var r TestResponse
			r.Error = code(err).Error()

			b, err := json.Marshal(r)
			if err != nil {
				t.Fatal(err)
			}

			if err := c.Publish(reply, b); err != nil {
				t.Fatal(err)
			}
		}),
	)

	resp := testRequest(t, c, handler)

	if want, have := errTeapot.Error(), resp.Error; want != have {
		t.Errorf("want %s, have %s", want, have)
	}
}

func TestSubscriberHappySubject(t *testing.T) {
	step, response := testSubscriber(t)
	step()
	r := <-response

	var resp TestResponse
	err := json.Unmarshal(r.Data, &resp)
	if err != nil {
		t.Fatal(err)
	}

	if want, have := "", resp.Error; want != have {
		t.Errorf("want %s, have %s (%s)", want, have, r.Data)
	}
}

func TestMultipleSubscriberBefore(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	var (
		response = struct{ Body string }{"go eat a fly ugly\n"}
		wg       sync.WaitGroup
		done     = make(chan struct{})
	)
	handler := natstransport.NewSubscriber(
		endpoint.Nop,
		func(context.Context, *nats.Msg) (interface{}, error) {
			return struct{}{}, nil
		},
		func(_ context.Context, reply string, nc *nats.Conn, _ interface{}) error {
			b, err := json.Marshal(response)
			if err != nil {
				return err
			}

			return c.Publish(reply, b)
		},
		natstransport.SubscriberBefore(func(ctx context.Context, _ *nats.Msg) context.Context {
			ctx = context.WithValue(ctx, "one", 1)

			return ctx
		}),
		natstransport.SubscriberBefore(func(ctx context.Context, _ *nats.Msg) context.Context {
			if _, ok := ctx.Value("one").(int); !ok {
				t.Error("Value was not set properly when multiple ServerBefores are used")
			}

			close(done)
			return ctx
		}),
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	wg.Add(1)
	go func() {
		defer wg.Done()
		_, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
		if err != nil {
			t.Fatal(err)
		}
	}()

	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for finalizer")
	}

	wg.Wait()
}

func TestMultipleSubscriberAfter(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	var (
		response = struct{ Body string }{"go eat a fly ugly\n"}
		wg       sync.WaitGroup
		done     = make(chan struct{})
	)
	handler := natstransport.NewSubscriber(
		endpoint.Nop,
		func(context.Context, *nats.Msg) (interface{}, error) {
			return struct{}{}, nil
		},
		func(_ context.Context, reply string, nc *nats.Conn, _ interface{}) error {
			b, err := json.Marshal(response)
			if err != nil {
				return err
			}
			return c.Publish(reply, b)
		},
		natstransport.SubscriberAfter(func(ctx context.Context, nc *nats.Conn) context.Context {
			return context.WithValue(ctx, "one", 1)
		}),
		natstransport.SubscriberAfter(func(ctx context.Context, nc *nats.Conn) context.Context {
			if _, ok := ctx.Value("one").(int); !ok {
				t.Error("Value was not set properly when multiple ServerAfters are used")
			}
			close(done)
			return ctx
		}),
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	wg.Add(1)
	go func() {
		defer wg.Done()
		_, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
		if err != nil {
			t.Fatal(err)
		}
	}()

	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for finalizer")
	}

	wg.Wait()
}

func TestSubscriberFinalizerFunc(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	var (
		response = struct{ Body string }{"go eat a fly ugly\n"}
		wg       sync.WaitGroup
		done     = make(chan struct{})
	)
	handler := natstransport.NewSubscriber(
		endpoint.Nop,
		func(context.Context, *nats.Msg) (interface{}, error) {
			return struct{}{}, nil
		},
		func(_ context.Context, reply string, nc *nats.Conn, _ interface{}) error {
			b, err := json.Marshal(response)
			if err != nil {
				return err
			}

			return c.Publish(reply, b)
		},
		natstransport.SubscriberFinalizer(func(ctx context.Context, _ *nats.Msg) {
			close(done)
		}),
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	wg.Add(1)
	go func() {
		defer wg.Done()
		_, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
		if err != nil {
			t.Fatal(err)
		}
	}()

	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for finalizer")
	}

	wg.Wait()
}

func TestEncodeJSONResponse(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) {
			return struct {
				Foo string `json:"foo"`
			}{"bar"}, nil
		},
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		natstransport.EncodeJSONResponse,
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
	if err != nil {
		t.Fatal(err)
	}

	if want, have := `{"foo":"bar"}`, strings.TrimSpace(string(r.Data)); want != have {
		t.Errorf("Body: want %s, have %s", want, have)
	}
}

type responseError struct {
	msg string
}

func (m responseError) Error() string {
	return m.msg
}

func TestErrorEncoder(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	errResp := struct {
		Error string `json:"err"`
	}{"oh no"}
	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) {
			return nil, responseError{msg: errResp.Error}
		},
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		natstransport.EncodeJSONResponse,
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
	if err != nil {
		t.Fatal(err)
	}

	b, err := json.Marshal(errResp)
	if err != nil {
		t.Fatal(err)
	}
	if string(b) != string(r.Data) {
		t.Errorf("ErrorEncoder: got: %q, expected: %q", r.Data, b)
	}
}

type noContentResponse struct{}

func TestEncodeNoContent(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(context.Context, interface{}) (interface{}, error) { return noContentResponse{}, nil },
		func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
		natstransport.EncodeJSONResponse,
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
	if err != nil {
		t.Fatal(err)
	}

	if want, have := `{}`, strings.TrimSpace(string(r.Data)); want != have {
		t.Errorf("Body: want %s, have %s", want, have)
	}
}

func TestNoOpRequestDecoder(t *testing.T) {
	s, c := newNATSConn(t)
	defer func() { s.Shutdown(); s.WaitForShutdown() }()
	defer c.Close()

	handler := natstransport.NewSubscriber(
		func(ctx context.Context, request interface{}) (interface{}, error) {
			if request != nil {
				t.Error("Expected nil request in endpoint when using NopRequestDecoder")
			}
			return nil, nil
		},
		natstransport.NopRequestDecoder,
		natstransport.EncodeJSONResponse,
	)

	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
	if err != nil {
		t.Fatal(err)
	}

	if want, have := `null`, strings.TrimSpace(string(r.Data)); want != have {
		t.Errorf("Body: want %s, have %s", want, have)
	}
}

func testSubscriber(t *testing.T) (step func(), resp <-chan *nats.Msg) {
	var (
		stepch   = make(chan bool)
		endpoint = func(context.Context, interface{}) (interface{}, error) {
			<-stepch
			return struct{}{}, nil
		}
		response = make(chan *nats.Msg)
		handler  = natstransport.NewSubscriber(
			endpoint,
			func(context.Context, *nats.Msg) (interface{}, error) { return struct{}{}, nil },
			natstransport.EncodeJSONResponse,
			natstransport.SubscriberBefore(func(ctx context.Context, msg *nats.Msg) context.Context { return ctx }),
			natstransport.SubscriberAfter(func(ctx context.Context, nc *nats.Conn) context.Context { return ctx }),
		)
	)

	go func() {
		s, c := newNATSConn(t)
		defer func() { s.Shutdown(); s.WaitForShutdown() }()
		defer c.Close()

		sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
		if err != nil {
			t.Fatal(err)
		}
		defer sub.Unsubscribe()

		r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
		if err != nil {
			t.Fatal(err)
		}

		response <- r
	}()

	return func() { stepch <- true }, response
}

func testRequest(t *testing.T, c *nats.Conn, handler *natstransport.Subscriber) TestResponse {
	sub, err := c.QueueSubscribe("natstransport.test", "natstransport", handler.ServeMsg(c))
	if err != nil {
		t.Fatal(err)
	}
	defer sub.Unsubscribe()

	r, err := c.Request("natstransport.test", []byte("test data"), 2*time.Second)
	if err != nil {
		t.Fatal(err)
	}

	var resp TestResponse
	err = json.Unmarshal(r.Data, &resp)
	if err != nil {
		t.Fatal(err)
	}

	return resp
}
