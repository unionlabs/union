package runtime_test

import (
	"bytes"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"google.golang.org/genproto/googleapis/api/httpbody"
	"google.golang.org/protobuf/encoding/protojson"
)

func TestHTTPBodyContentType(t *testing.T) {
	m := runtime.HTTPBodyMarshaler{
		&runtime.JSONPb{
			MarshalOptions: protojson.MarshalOptions{
				UseProtoNames: true,
			},
		},
	}
	expected := "CustomContentType"
	message := &httpbody.HttpBody{
		ContentType: expected,
	}
	res := m.ContentType(nil)
	if res != "application/json" {
		t.Errorf("content type not equal (%q, %q)", res, expected)
	}
	res = m.ContentType(message)
	if res != expected {
		t.Errorf("content type not equal (%q, %q)", res, expected)
	}
}

func TestHTTPBodyMarshal(t *testing.T) {
	m := runtime.HTTPBodyMarshaler{
		&runtime.JSONPb{
			MarshalOptions: protojson.MarshalOptions{
				UseProtoNames: true,
			},
		},
	}
	expected := []byte("Some test")
	message := &httpbody.HttpBody{
		Data: expected,
	}
	res, err := m.Marshal(message)
	if err != nil {
		t.Errorf("m.Marshal(%#v) failed with %v; want success", message, err)
	}
	if !bytes.Equal(res, expected) {
		t.Errorf("Marshalled data not equal (%q, %q)", res, expected)

	}
}
