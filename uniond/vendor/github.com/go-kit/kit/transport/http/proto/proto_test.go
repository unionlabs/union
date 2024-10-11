package proto

import (
	"context"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"testing"

	"google.golang.org/protobuf/proto"
)

func TestEncodeProtoRequest(t *testing.T) {
	cat := &Cat{Name: "Ziggy", Age: 13, Breed: "Lumpy"}

	r := httptest.NewRequest(http.MethodGet, "/cat", nil)

	err := EncodeProtoRequest(context.TODO(), r, cat)
	if err != nil {
		t.Errorf("expected no encoding errors but got: %s", err)
		return
	}

	const xproto = "application/x-protobuf"
	if typ := r.Header.Get("Content-Type"); typ != xproto {
		t.Errorf("expected content type of %q, got %q", xproto, typ)
		return
	}

	bod, err := ioutil.ReadAll(r.Body)
	if err != nil {
		t.Errorf("expected no read errors but got: %s", err)
		return
	}
	defer r.Body.Close()

	var got Cat
	err = proto.Unmarshal(bod, &got)
	if err != nil {
		t.Errorf("expected no proto errors but got: %s", err)
		return
	}

	if !proto.Equal(&got, cat) {
		t.Errorf("expected cats to be equal but got:\n\n%#v\n\nwant:\n\n%#v", got, cat)
		return
	}
}

func TestEncodeProtoResponse(t *testing.T) {
	cat := &Cat{Name: "Ziggy", Age: 13, Breed: "Lumpy"}

	wr := httptest.NewRecorder()

	err := EncodeProtoResponse(context.TODO(), wr, cat)
	if err != nil {
		t.Errorf("expected no encoding errors but got: %s", err)
		return
	}

	w := wr.Result()

	const xproto = "application/x-protobuf"
	if typ := w.Header.Get("Content-Type"); typ != xproto {
		t.Errorf("expected content type of %q, got %q", xproto, typ)
		return
	}

	if w.StatusCode != http.StatusTeapot {
		t.Errorf("expected status code of %d, got %d", http.StatusTeapot, w.StatusCode)
		return
	}

	bod, err := ioutil.ReadAll(w.Body)
	if err != nil {
		t.Errorf("expected no read errors but got: %s", err)
		return
	}
	defer w.Body.Close()

	var got Cat
	err = proto.Unmarshal(bod, &got)
	if err != nil {
		t.Errorf("expected no proto errors but got: %s", err)
		return
	}

	if !proto.Equal(&got, cat) {
		t.Errorf("expected cats to be equal but got:\n\n%#v\n\nwant:\n\n%#v", got, cat)
		return
	}
}

func (c *Cat) StatusCode() int {
	return http.StatusTeapot
}
