package handlers

import (
	"bytes"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestRecoveryLoggerWithDefaultOptions(t *testing.T) {
	var buf bytes.Buffer
	log.SetOutput(&buf)

	handler := RecoveryHandler()
	handlerFunc := http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		panic("Unexpected error!")
	})

	recovery := handler(handlerFunc)
	recovery.ServeHTTP(httptest.NewRecorder(), newRequest(http.MethodGet, "/subdir/asdf"))

	if !strings.Contains(buf.String(), "Unexpected error!") {
		t.Fatalf("Got log %#v, wanted substring %#v", buf.String(), "Unexpected error!")
	}
}

func TestRecoveryLoggerWithCustomLogger(t *testing.T) {
	var buf bytes.Buffer
	logger := log.New(&buf, "", log.LstdFlags)

	handlerFunc := http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		panic("Unexpected error!")
	})

	t.Run("Without print stack", func(t *testing.T) {
		handler := RecoveryHandler(RecoveryLogger(logger), PrintRecoveryStack(false))

		recovery := handler(handlerFunc)
		recovery.ServeHTTP(httptest.NewRecorder(), newRequest(http.MethodGet, "/subdir/asdf"))

		if !strings.Contains(buf.String(), "Unexpected error!") {
			t.Fatalf("Got log %#v, wanted substring %#v", buf.String(), "Unexpected error!")
		}
	})

	t.Run("With print stack enabled", func(t *testing.T) {
		handler := RecoveryHandler(RecoveryLogger(logger), PrintRecoveryStack(true))

		recovery := handler(handlerFunc)
		recovery.ServeHTTP(httptest.NewRecorder(), newRequest(http.MethodGet, "/subdir/asdf"))

		if !strings.Contains(buf.String(), "runtime/debug.Stack") {
			t.Fatalf("Got log %#v, wanted substring %#v", buf.String(), "runtime/debug.Stack")
		}
	})
}
