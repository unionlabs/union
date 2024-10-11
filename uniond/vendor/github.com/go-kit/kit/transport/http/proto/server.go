package proto

import (
	"context"
	"errors"
	"net/http"

	httptransport "github.com/go-kit/kit/transport/http"
	"google.golang.org/protobuf/proto"
)

// EncodeProtoResponse is an EncodeResponseFunc that serializes the response as Protobuf.
// Many Proto-over-HTTP services can use it as a sensible default. If the response
// implements Headerer, the provided headers will be applied to the response. If the
// response implements StatusCoder, the provided StatusCode will be used instead of 200.
func EncodeProtoResponse(ctx context.Context, w http.ResponseWriter, pres interface{}) error {
	res, ok := pres.(proto.Message)
	if !ok {
		return errors.New("response does not implement proto.Message")
	}
	w.Header().Set("Content-Type", "application/x-protobuf")
	if headerer, ok := w.(httptransport.Headerer); ok {
		for k := range headerer.Headers() {
			w.Header().Set(k, headerer.Headers().Get(k))
		}
	}
	code := http.StatusOK
	if sc, ok := pres.(httptransport.StatusCoder); ok {
		code = sc.StatusCode()
	}
	w.WriteHeader(code)
	if code == http.StatusNoContent {
		return nil
	}
	if res == nil {
		return nil
	}
	b, err := proto.Marshal(res)
	if err != nil {
		return err
	}
	_, err = w.Write(b)
	if err != nil {
		return err
	}
	return nil
}
