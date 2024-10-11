package etcd

import (
	"bytes"
	"errors"
	"testing"

	"github.com/go-kit/log"
)

// testClient is a basic implementation of Client
type testClient struct {
	registerRes error // value returned when Register or Deregister is called
}

func (tc *testClient) GetEntries(prefix string) ([]string, error) {
	return nil, nil
}

func (tc *testClient) WatchPrefix(prefix string, ch chan struct{}) {
	return
}

func (tc *testClient) Register(s Service) error {
	return tc.registerRes
}

func (tc *testClient) Deregister(s Service) error {
	return tc.registerRes
}

// default service used to build registrar in our tests
var testService = Service{"testKey", "testValue", nil, nil}

// NewRegistar should return a registar with a logger using the service key and value
func TestNewRegistar(t *testing.T) {
	c := Client(&testClient{nil})
	buf := &bytes.Buffer{}
	logger := log.NewLogfmtLogger(buf)
	r := NewRegistrar(
		c,
		testService,
		logger,
	)

	if err := r.logger.Log("msg", "message"); err != nil {
		t.Fatal(err)
	}
	if want, have := "key=testKey value=testValue msg=message\n", buf.String(); want != have {
		t.Errorf("\nwant: %shave: %s", want, have)
	}
}

// Register log the error returned by the client or log the successful registration action
// table of test cases for method Register
var registerTestTable = []struct {
	registerRes error  // value returned by the client on calls to Register
	log         string // expected log by the registrar

}{
	// test case: an error is returned by the client
	{errors.New("regError"), "key=testKey value=testValue err=regError\n"},
	// test case: registration successful
	{nil, "key=testKey value=testValue action=register\n"},
}

func TestRegister(t *testing.T) {
	for _, tc := range registerTestTable {
		c := Client(&testClient{tc.registerRes})
		buf := &bytes.Buffer{}
		logger := log.NewLogfmtLogger(buf)
		r := NewRegistrar(
			c,
			testService,
			logger,
		)
		r.Register()
		if want, have := tc.log, buf.String(); want != have {
			t.Fatalf("want %v, have %v", want, have)
		}
	}
}

// Deregister log the error returned by the client or log the successful deregistration action
// table of test cases for method Deregister
var deregisterTestTable = []struct {
	deregisterRes error  // value returned by the client on calls to Deregister
	log           string // expected log by the registrar
}{
	// test case: an error is returned by the client
	{errors.New("deregError"), "key=testKey value=testValue err=deregError\n"},
	// test case: deregistration successful
	{nil, "key=testKey value=testValue action=deregister\n"},
}

func TestDeregister(t *testing.T) {
	for _, tc := range deregisterTestTable {
		c := Client(&testClient{tc.deregisterRes})
		buf := &bytes.Buffer{}
		logger := log.NewLogfmtLogger(buf)
		r := NewRegistrar(
			c,
			testService,
			logger,
		)
		r.Deregister()
		if want, have := tc.log, buf.String(); want != have {
			t.Fatalf("want %v, have %v", want, have)
		}
	}
}
