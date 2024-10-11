package proto

import (
	"bytes"
	"context"
	"errors"
	"io/ioutil"
	"net/http"

	httptransport "github.com/go-kit/kit/transport/http"
	"google.golang.org/protobuf/proto"
)

// EncodeProtoRequest is an EncodeRequestFunc that serializes the request as Protobuf.
// If the request implements Headerer, the provided headers will be applied
// to the request. If the given request does not implement proto.Message, an error will
// be returned.
func EncodeProtoRequest(_ context.Context, r *http.Request, preq interface{}) error {
	r.Header.Set("Content-Type", "application/x-protobuf")
	if headerer, ok := preq.(httptransport.Headerer); ok {
		for k := range headerer.Headers() {
			r.Header.Set(k, headerer.Headers().Get(k))
		}
	}
	req, ok := preq.(proto.Message)
	if !ok {
		return errors.New("response does not implement proto.Message")
	}

	b, err := proto.Marshal(req)
	if err != nil {
		return err
	}
	r.ContentLength = int64(len(b))
	r.Body = ioutil.NopCloser(bytes.NewReader(b))
	return nil
}
