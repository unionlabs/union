package zk

import (
	"bytes"
	"testing"
	"time"

	stdzk "github.com/go-zookeeper/zk"

	"github.com/go-kit/log"
)

func TestNewClient(t *testing.T) {
	var (
		acl            = stdzk.WorldACL(stdzk.PermRead)
		connectTimeout = 3 * time.Second
		sessionTimeout = 20 * time.Second
		payload        = [][]byte{[]byte("Payload"), []byte("Test")}
	)

	c, err := NewClient(
		[]string{"FailThisInvalidHost!!!"},
		log.NewNopLogger(),
	)
	if err == nil {
		t.Errorf("expected error, got nil")
	}

	hasFired := false
	calledEventHandler := make(chan struct{})
	eventHandler := func(event stdzk.Event) {
		if !hasFired {
			// test is successful if this function has fired at least once
			hasFired = true
			close(calledEventHandler)
		}
	}

	c, err = NewClient(
		[]string{"localhost"},
		log.NewNopLogger(),
		ACL(acl),
		ConnectTimeout(connectTimeout),
		SessionTimeout(sessionTimeout),
		Payload(payload),
		EventHandler(eventHandler),
	)
	if err != nil {
		t.Fatal(err)
	}
	defer c.Stop()

	clientImpl, ok := c.(*client)
	if !ok {
		t.Fatal("retrieved incorrect Client implementation")
	}
	if want, have := acl, clientImpl.acl; want[0] != have[0] {
		t.Errorf("want %+v, have %+v", want, have)
	}
	if want, have := connectTimeout, clientImpl.connectTimeout; want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	if want, have := sessionTimeout, clientImpl.sessionTimeout; want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	if want, have := payload, clientImpl.rootNodePayload; !bytes.Equal(want[0], have[0]) || !bytes.Equal(want[1], have[1]) {
		t.Errorf("want %s, have %s", want, have)
	}

	select {
	case <-calledEventHandler:
	case <-time.After(100 * time.Millisecond):
		t.Errorf("event handler never called")
	}
}

func TestOptions(t *testing.T) {
	_, err := NewClient([]string{"localhost"}, log.NewNopLogger(), Credentials("valid", "credentials"))
	if err != nil && err != stdzk.ErrNoServer {
		t.Errorf("unexpected error: %v", err)
	}

	_, err = NewClient([]string{"localhost"}, log.NewNopLogger(), Credentials("nopass", ""))
	if want, have := err, ErrInvalidCredentials; want != have {
		t.Errorf("want %v, have %v", want, have)
	}

	_, err = NewClient([]string{"localhost"}, log.NewNopLogger(), ConnectTimeout(0))
	if err == nil {
		t.Errorf("expected connect timeout error")
	}

	_, err = NewClient([]string{"localhost"}, log.NewNopLogger(), SessionTimeout(0))
	if err == nil {
		t.Errorf("expected connect timeout error")
	}
}

func TestCreateParentNodes(t *testing.T) {
	payload := [][]byte{[]byte("Payload"), []byte("Test")}

	c, err := NewClient([]string{"localhost:65500"}, log.NewNopLogger())
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}
	if c == nil {
		t.Fatal("expected new Client, got nil")
	}

	s, err := NewInstancer(c, "/validpath", log.NewNopLogger())
	if err != stdzk.ErrNoServer {
		t.Errorf("unexpected error: %v", err)
	}
	if s != nil {
		t.Error("expected failed new Instancer")
	}

	s, err = NewInstancer(c, "invalidpath", log.NewNopLogger())
	if err != stdzk.ErrInvalidPath {
		t.Errorf("unexpected error: %v", err)
	}
	_, _, err = c.GetEntries("/validpath")
	if err != stdzk.ErrNoServer {
		t.Errorf("unexpected error: %v", err)
	}

	c.Stop()

	err = c.CreateParentNodes("/validpath")
	if err != ErrClientClosed {
		t.Errorf("unexpected error: %v", err)
	}

	s, err = NewInstancer(c, "/validpath", log.NewNopLogger())
	if err != ErrClientClosed {
		t.Errorf("unexpected error: %v", err)
	}
	if s != nil {
		t.Error("expected failed new Instancer")
	}

	c, err = NewClient([]string{"localhost:65500"}, log.NewNopLogger(), Payload(payload))
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}
	if c == nil {
		t.Fatal("expected new Client, got nil")
	}

	s, err = NewInstancer(c, "/validpath", log.NewNopLogger())
	if err != stdzk.ErrNoServer {
		t.Errorf("unexpected error: %v", err)
	}
	if s != nil {
		t.Error("expected failed new Instancer")
	}
}
