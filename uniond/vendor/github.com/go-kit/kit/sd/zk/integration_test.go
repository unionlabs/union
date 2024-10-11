// +build integration

package zk

import (
	"bytes"
	"os"
	"testing"
	"time"

	stdzk "github.com/go-zookeeper/zk"
)

var (
	host []string
)

func TestMain(m *testing.M) {
	zkAddr := os.Getenv("ZK_ADDR")
	if zkAddr != "" {
		host = []string{zkAddr}
	}
	m.Run()
}

func TestCreateParentNodesOnServer(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	payload := [][]byte{[]byte("Payload"), []byte("Test")}
	c1, err := NewClient(host, logger, Payload(payload))
	if err != nil {
		t.Fatalf("Connect returned error: %v", err)
	}
	if c1 == nil {
		t.Fatal("Expected pointer to client, got nil")
	}
	defer c1.Stop()

	instancer, err := NewInstancer(c1, path, logger)
	if err != nil {
		t.Fatalf("Unable to create Subscriber: %v", err)
	}
	defer instancer.Stop()

	state := instancer.state()
	if state.Err != nil {
		t.Fatal(err)
	}
	if want, have := 0, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	c2, err := NewClient(host, logger)
	if err != nil {
		t.Fatalf("Connect returned error: %v", err)
	}
	defer c2.Stop()
	data, _, err := c2.(*client).Get(path)
	if err != nil {
		t.Fatal(err)
	}
	// test Client implementation of CreateParentNodes. It should have created
	// our payload
	if bytes.Compare(data, payload[1]) != 0 {
		t.Errorf("want %s, have %s", payload[1], data)
	}

}

func TestCreateBadParentNodesOnServer(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	c, _ := NewClient(host, logger)
	defer c.Stop()

	_, err := NewInstancer(c, "invalid/path", logger)

	if want, have := stdzk.ErrInvalidPath, err; want != have {
		t.Errorf("want %v, have %v", want, have)
	}
}

func TestCredentials1(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	acl := stdzk.DigestACL(stdzk.PermAll, "user", "secret")
	c, _ := NewClient(host, logger, ACL(acl), Credentials("user", "secret"))
	defer c.Stop()

	_, err := NewInstancer(c, "/acl-issue-test", logger)

	if err != nil {
		t.Fatal(err)
	}
}

func TestCredentials2(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	acl := stdzk.DigestACL(stdzk.PermAll, "user", "secret")
	c, _ := NewClient(host, logger, ACL(acl))
	defer c.Stop()

	_, err := NewInstancer(c, "/acl-issue-test", logger)

	if err != stdzk.ErrNoAuth {
		t.Errorf("want %v, have %v", stdzk.ErrNoAuth, err)
	}
}

func TestConnection(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	c, _ := NewClient(host, logger)
	c.Stop()

	_, err := NewInstancer(c, "/acl-issue-test", logger)

	if err != ErrClientClosed {
		t.Errorf("want %v, have %v", ErrClientClosed, err)
	}
}

func TestGetEntriesOnServer(t *testing.T) {
	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}
	var instancePayload = "10.0.3.204:8002"

	c1, err := NewClient(host, logger)
	if err != nil {
		t.Fatalf("Connect returned error: %v", err)
	}

	defer c1.Stop()

	c2, err := NewClient(host, logger)
	s, err := NewInstancer(c2, path, logger)
	if err != nil {
		t.Fatal(err)
	}
	defer c2.Stop()

	instance1 := &Service{
		Path: path,
		Name: "instance1",
		Data: []byte(instancePayload),
	}
	if err = c2.Register(instance1); err != nil {
		t.Fatalf("Unable to create test ephemeral znode 1: %+v", err)
	}
	instance2 := &Service{
		Path: path,
		Name: "instance2",
		Data: []byte(instancePayload),
	}
	if err = c2.Register(instance2); err != nil {
		t.Fatalf("Unable to create test ephemeral znode 2: %+v", err)
	}

	time.Sleep(50 * time.Millisecond)

	state := s.state()
	if state.Err != nil {
		t.Fatal(state.Err)
	}
	if want, have := 2, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestGetEntriesPayloadOnServer(t *testing.T) {
	t.Skip("FLAKY")

	if len(host) == 0 {
		t.Skip("ZK_ADDR not set; skipping integration test")
	}

	c, err := NewClient(host, logger)
	if err != nil {
		t.Fatalf("Connect returned error: %v", err)
	}

	_, eventc, err := c.GetEntries(path)
	if err != nil {
		t.Fatal(err)
	}

	instance3 := Service{
		Path: path,
		Name: "instance3",
		Data: []byte("just some payload"),
	}

	registrar := NewRegistrar(c, instance3, logger)
	registrar.Register()

	select {
	case event := <-eventc:
		if want, have := stdzk.EventNodeChildrenChanged.String(), event.Type.String(); want != have {
			t.Errorf("want %s, have %s", want, have)
		}
	case <-time.After(10 * time.Second):
		t.Errorf("expected incoming watch event, timeout occurred")
	}

	_, eventc, err = c.GetEntries(path)
	if err != nil {
		t.Fatal(err)
	}

	registrar.Deregister()
	select {
	case event := <-eventc:
		if want, have := stdzk.EventNodeChildrenChanged.String(), event.Type.String(); want != have {
			t.Errorf("want %s, have %s", want, have)
		}
	case <-time.After(100 * time.Millisecond):
		t.Errorf("expected incoming watch event, timeout occurred")
	}

}
