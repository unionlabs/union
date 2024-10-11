package log_test

import (
	"bytes"
	"io"
	"os"
	"testing"

	"github.com/go-kit/log"
)

func TestSwapLogger(t *testing.T) {
	t.Parallel()
	var logger log.SwapLogger

	// Zero value does not panic or error.
	err := logger.Log("k", "v")
	if got, want := err, error(nil); got != want {
		t.Errorf("got %v, want %v", got, want)
	}

	buf := &bytes.Buffer{}
	json := log.NewJSONLogger(buf)
	logger.Swap(json)

	if err := logger.Log("k", "v"); err != nil {
		t.Error(err)
	}
	if got, want := buf.String(), `{"k":"v"}`+"\n"; got != want {
		t.Errorf("got %v, want %v", got, want)
	}

	buf.Reset()
	prefix := log.NewLogfmtLogger(buf)
	logger.Swap(prefix)

	if err := logger.Log("k", "v"); err != nil {
		t.Error(err)
	}
	if got, want := buf.String(), "k=v\n"; got != want {
		t.Errorf("got %v, want %v", got, want)
	}

	buf.Reset()
	logger.Swap(nil)

	if err := logger.Log("k", "v"); err != nil {
		t.Error(err)
	}
	if got, want := buf.String(), ""; got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestSwapLoggerConcurrency(t *testing.T) {
	t.Parallel()
	testConcurrency(t, &log.SwapLogger{}, 10000)
}

func TestSyncLoggerConcurrency(t *testing.T) {
	var w io.Writer //lint:ignore S1021 I prefer this
	w = &bytes.Buffer{}
	logger := log.NewLogfmtLogger(w)
	logger = log.NewSyncLogger(logger)
	testConcurrency(t, logger, 10000)
}

func TestSyncWriterConcurrency(t *testing.T) {
	var w io.Writer
	w = &bytes.Buffer{}
	w = log.NewSyncWriter(w)
	testConcurrency(t, log.NewLogfmtLogger(w), 10000)
}

func TestSyncWriterFd(t *testing.T) {
	_, ok := log.NewSyncWriter(os.Stdout).(interface {
		Fd() uintptr
	})

	if !ok {
		t.Error("NewSyncWriter does not pass through Fd method")
	}
}

func TestSyncLoggerPanic(t *testing.T) {
	var logger log.Logger
	logger = log.LoggerFunc(func(...interface{}) error { panic("!") })
	logger = log.NewSyncLogger(logger)

	f := func() {
		defer func() {
			if x := recover(); x != nil {
				t.Log(x)
			}
		}()
		logger.Log("hello", "world")
	}

	f()
	f() // without defer Unlock, this one can deadlock
}
