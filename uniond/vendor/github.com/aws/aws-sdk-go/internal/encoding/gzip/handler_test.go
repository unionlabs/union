package gzip

import (
	"bytes"
	"io/ioutil"
	"net/http"
	"strconv"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws/request"
)

func TestGzipRequestHandler(t *testing.T) {
	handler := NewGzipRequestHandler()

	req := &request.Request{}
	uncompressed := "asdfasdfasdf"
	req.Body = strings.NewReader(uncompressed)
	httpReq, err := http.NewRequest("POST", "http://localhost", strings.NewReader(uncompressed))
	if err != nil {
		panic(err)
	}
	req.HTTPRequest = httpReq

	expectCompressed, err := compress(strings.NewReader(uncompressed))
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	handler.Fn(req)
	if req.Error != nil {
		t.Fatalf("expect no error, got %v", req.Error)
	}

	if e, a := "gzip", req.HTTPRequest.Header.Get("Content-Encoding"); e != a {
		t.Errorf("expect %v content-encoding, got %v", e, a)
	}
	if e, a := strconv.Itoa(len(expectCompressed)), req.HTTPRequest.Header.Get("Content-Length"); e != a {
		t.Errorf("expect %v content-length, got %v", e, a)
	}

	actualCompressed, err := ioutil.ReadAll(req.Body)
	if err != nil {
		t.Fatalf("ReadAll request body failed, %v", err)
	}
	if !bytes.Equal(expectCompressed, actualCompressed) {
		t.Errorf("expect new body to equal expectCompressed")
	}
}
