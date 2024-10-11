package etcdv3

import (
	"errors"
	"testing"

	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

var _ sd.Instancer = (*Instancer)(nil) // API check

type testKV struct {
	Key   []byte
	Value []byte
}

type testResponse struct {
	Kvs []testKV
}

var (
	fakeResponse = testResponse{
		Kvs: []testKV{
			{
				Key:   []byte("/foo/1"),
				Value: []byte("1:1"),
			},
			{
				Key:   []byte("/foo/2"),
				Value: []byte("2:2"),
			},
		},
	}
)

var _ sd.Instancer = &Instancer{} // API check

func TestInstancer(t *testing.T) {
	client := &fakeClient{
		responses: map[string]testResponse{"/foo": fakeResponse},
	}

	s, err := NewInstancer(client, "/foo", log.NewNopLogger())
	if err != nil {
		t.Fatal(err)
	}
	defer s.Stop()

	if state := s.cache.State(); state.Err != nil {
		t.Fatal(state.Err)
	}
}

type fakeClient struct {
	responses map[string]testResponse
}

func (c *fakeClient) GetEntries(prefix string) ([]string, error) {
	response, ok := c.responses[prefix]
	if !ok {
		return nil, errors.New("key not exist")
	}

	entries := make([]string, len(response.Kvs))
	for i, node := range response.Kvs {
		entries[i] = string(node.Value)
	}
	return entries, nil
}

func (c *fakeClient) WatchPrefix(prefix string, ch chan struct{}) {
}

func (c *fakeClient) LeaseID() int64 {
	return 0
}

func (c *fakeClient) Register(Service) error {
	return nil
}
func (c *fakeClient) Deregister(Service) error {
	return nil
}
