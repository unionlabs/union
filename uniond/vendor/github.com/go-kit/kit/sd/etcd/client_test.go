package etcd

import (
	"context"
	"errors"
	"reflect"
	"testing"
	"time"

	etcd "go.etcd.io/etcd/client/v2"
)

func TestNewClient(t *testing.T) {
	client, err := NewClient(
		context.Background(),
		[]string{"http://irrelevant:12345"},
		ClientOptions{
			DialTimeout:             2 * time.Second,
			DialKeepAlive:           2 * time.Second,
			HeaderTimeoutPerRequest: 2 * time.Second,
		},
	)
	if err != nil {
		t.Fatalf("unexpected error creating client: %v", err)
	}
	if client == nil {
		t.Fatal("expected new Client, got nil")
	}
}

// NewClient should fail when providing invalid or missing endpoints.
func TestOptions(t *testing.T) {
	a, err := NewClient(
		context.Background(),
		[]string{},
		ClientOptions{
			Cert:                    "",
			Key:                     "",
			CACert:                  "",
			DialTimeout:             2 * time.Second,
			DialKeepAlive:           2 * time.Second,
			HeaderTimeoutPerRequest: 2 * time.Second,
		},
	)
	if err == nil {
		t.Errorf("expected error: %v", err)
	}
	if a != nil {
		t.Fatalf("expected client to be nil on failure")
	}

	_, err = NewClient(
		context.Background(),
		[]string{"http://irrelevant:12345"},
		ClientOptions{
			Cert:                    "blank.crt",
			Key:                     "blank.key",
			CACert:                  "blank.CACert",
			DialTimeout:             2 * time.Second,
			DialKeepAlive:           2 * time.Second,
			HeaderTimeoutPerRequest: 2 * time.Second,
		},
	)
	if err == nil {
		t.Errorf("expected error: %v", err)
	}
}

// Mocks of the underlying etcd.KeysAPI interface that is called by the methods we want to test

// fakeKeysAPI implements etcd.KeysAPI, event and err are channels used to emulate
// an etcd event or error, getres will be returned when etcd.KeysAPI.Get is called.
type fakeKeysAPI struct {
	event  chan bool
	err    chan bool
	getres *getResult
}

type getResult struct {
	resp *etcd.Response
	err  error
}

// Get return the content of getres or nil, nil
func (fka *fakeKeysAPI) Get(ctx context.Context, key string, opts *etcd.GetOptions) (*etcd.Response, error) {
	if fka.getres == nil {
		return nil, nil
	}
	return fka.getres.resp, fka.getres.err
}

// Set is not used in the tests
func (fka *fakeKeysAPI) Set(ctx context.Context, key, value string, opts *etcd.SetOptions) (*etcd.Response, error) {
	return nil, nil
}

// Delete is not used in the tests
func (fka *fakeKeysAPI) Delete(ctx context.Context, key string, opts *etcd.DeleteOptions) (*etcd.Response, error) {
	return nil, nil
}

// Create is not used in the tests
func (fka *fakeKeysAPI) Create(ctx context.Context, key, value string) (*etcd.Response, error) {
	return nil, nil
}

// CreateInOrder is not used in the tests
func (fka *fakeKeysAPI) CreateInOrder(ctx context.Context, dir, value string, opts *etcd.CreateInOrderOptions) (*etcd.Response, error) {
	return nil, nil
}

// Update is not used in the tests
func (fka *fakeKeysAPI) Update(ctx context.Context, key, value string) (*etcd.Response, error) {
	return nil, nil
}

// Watcher return a fakeWatcher that will forward event and error received on the channels
func (fka *fakeKeysAPI) Watcher(key string, opts *etcd.WatcherOptions) etcd.Watcher {
	return &fakeWatcher{fka.event, fka.err}
}

// fakeWatcher implements etcd.Watcher
type fakeWatcher struct {
	event chan bool
	err   chan bool
}

// Next blocks until an etcd event or error is emulated.
// When an event occurs it just return nil response and error.
// When an error occur it return a non nil error.
func (fw *fakeWatcher) Next(context.Context) (*etcd.Response, error) {
	select {
	case <-fw.event:
		return nil, nil
	case <-fw.err:
		return nil, errors.New("error from underlying etcd watcher")

	}
}

// newFakeClient return a new etcd.Client built on top of the mocked interfaces
func newFakeClient(event, err chan bool, getres *getResult) Client {
	return &client{
		keysAPI: &fakeKeysAPI{event, err, getres},
		ctx:     context.Background(),
	}
}

// Register should fail when the provided service has an empty key or value
func TestRegisterClient(t *testing.T) {
	client := newFakeClient(nil, nil, nil)

	err := client.Register(Service{Key: "", Value: "value", DeleteOptions: nil})
	if want, have := ErrNoKey, err; want != have {
		t.Fatalf("want %v, have %v", want, have)
	}

	err = client.Register(Service{Key: "key", Value: "", DeleteOptions: nil})
	if want, have := ErrNoValue, err; want != have {
		t.Fatalf("want %v, have %v", want, have)
	}

	err = client.Register(Service{Key: "key", Value: "value", DeleteOptions: nil})
	if err != nil {
		t.Fatal(err)
	}
}

// Deregister should fail if the input service has an empty key
func TestDeregisterClient(t *testing.T) {
	client := newFakeClient(nil, nil, nil)

	err := client.Deregister(Service{Key: "", Value: "value", DeleteOptions: nil})
	if want, have := ErrNoKey, err; want != have {
		t.Fatalf("want %v, have %v", want, have)
	}

	err = client.Deregister(Service{Key: "key", Value: "", DeleteOptions: nil})
	if err != nil {
		t.Fatal(err)
	}
}

// WatchPrefix notify the caller by writing on the channel if an etcd event occurs
// or return in case of an underlying error
func TestWatchPrefix(t *testing.T) {
	err := make(chan bool)
	event := make(chan bool)
	watchPrefixReturned := make(chan bool, 1)
	client := newFakeClient(event, err, nil)

	ch := make(chan struct{})
	go func() {
		client.WatchPrefix("prefix", ch) // block until an etcd event or error occurs
		watchPrefixReturned <- true
	}()

	// WatchPrefix force the caller to read once from the channel before actually
	// sending notification, emulate that first read.
	<-ch

	// Emulate an etcd event
	event <- true
	if want, have := struct{}{}, <-ch; want != have {
		t.Fatalf("want %v, have %v", want, have)
	}

	// Emulate an error, WatchPrefix should return
	err <- true
	select {
	case <-watchPrefixReturned:
		break
	case <-time.After(1 * time.Second):
		t.Fatal("WatchPrefix not returning on errors")
	}
}

var errKeyAPI = errors.New("emulate error returned by KeysAPI.Get")

// table of test cases for method GetEntries
var getEntriesTestTable = []struct {
	input getResult // value returned by the underlying etcd.KeysAPI.Get
	resp  []string  // response expected in output of GetEntries
	err   error     //error expected in output of GetEntries

}{
	// test case: an error is returned by etcd.KeysAPI.Get
	{getResult{nil, errKeyAPI}, nil, errKeyAPI},
	// test case: return a single leaf node, with an empty value
	{getResult{&etcd.Response{
		Action: "get",
		Node: &etcd.Node{
			Key:           "nodekey",
			Dir:           false,
			Value:         "",
			Nodes:         nil,
			CreatedIndex:  0,
			ModifiedIndex: 0,
			Expiration:    nil,
			TTL:           0,
		},
		PrevNode: nil,
		Index:    0,
	}, nil}, []string{}, nil},
	// test case: return a single leaf node, with a value
	{getResult{&etcd.Response{
		Action: "get",
		Node: &etcd.Node{
			Key:           "nodekey",
			Dir:           false,
			Value:         "nodevalue",
			Nodes:         nil,
			CreatedIndex:  0,
			ModifiedIndex: 0,
			Expiration:    nil,
			TTL:           0,
		},
		PrevNode: nil,
		Index:    0,
	}, nil}, []string{"nodevalue"}, nil},
	// test case: return a node with two childs
	{getResult{&etcd.Response{
		Action: "get",
		Node: &etcd.Node{
			Key:   "nodekey",
			Dir:   true,
			Value: "nodevalue",
			Nodes: []*etcd.Node{
				{
					Key:           "childnode1",
					Dir:           false,
					Value:         "childvalue1",
					Nodes:         nil,
					CreatedIndex:  0,
					ModifiedIndex: 0,
					Expiration:    nil,
					TTL:           0,
				},
				{
					Key:           "childnode2",
					Dir:           false,
					Value:         "childvalue2",
					Nodes:         nil,
					CreatedIndex:  0,
					ModifiedIndex: 0,
					Expiration:    nil,
					TTL:           0,
				},
			},
			CreatedIndex:  0,
			ModifiedIndex: 0,
			Expiration:    nil,
			TTL:           0,
		},
		PrevNode: nil,
		Index:    0,
	}, nil}, []string{"childvalue1", "childvalue2"}, nil},
}

func TestGetEntries(t *testing.T) {
	for _, et := range getEntriesTestTable {
		client := newFakeClient(nil, nil, &et.input)
		resp, err := client.GetEntries("prefix")
		if want, have := et.resp, resp; !reflect.DeepEqual(want, have) {
			t.Fatalf("want %v, have %v", want, have)
		}
		if want, have := et.err, err; want != have {
			t.Fatalf("want %v, have %v", want, have)
		}
	}
}
