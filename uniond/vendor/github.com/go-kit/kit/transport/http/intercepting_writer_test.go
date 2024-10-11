package http

import (
	"bufio"
	"io"
	"net"
	"net/http"
	"testing"
)

type versatileWriter struct {
	http.ResponseWriter
	closeNotifyCalled bool
	hijackCalled      bool
	readFromCalled    bool
	pushCalled        bool
	flushCalled       bool
}

func (v *versatileWriter) Flush() { v.flushCalled = true }
func (v *versatileWriter) Push(target string, opts *http.PushOptions) error {
	v.pushCalled = true
	return nil
}
func (v *versatileWriter) ReadFrom(r io.Reader) (n int64, err error) {
	v.readFromCalled = true
	return 0, nil
}
func (v *versatileWriter) CloseNotify() <-chan bool { v.closeNotifyCalled = true; return nil }
func (v *versatileWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	v.hijackCalled = true
	return nil, nil, nil
}

func TestInterceptingWriter_passthroughs(t *testing.T) {
	w := &versatileWriter{}
	iw := (&interceptingWriter{ResponseWriter: w}).reimplementInterfaces()
	iw.(http.Flusher).Flush()
	iw.(http.Pusher).Push("", nil)
	iw.(http.CloseNotifier).CloseNotify()
	iw.(http.Hijacker).Hijack()
	iw.(io.ReaderFrom).ReadFrom(nil)

	if !w.flushCalled {
		t.Error("Flush not called")
	}
	if !w.pushCalled {
		t.Error("Push not called")
	}
	if !w.closeNotifyCalled {
		t.Error("CloseNotify not called")
	}
	if !w.hijackCalled {
		t.Error("Hijack not called")
	}
	if !w.readFromCalled {
		t.Error("ReadFrom not called")
	}
}

// TestInterceptingWriter_reimplementInterfaces is also derived from
// https://github.com/felixge/httpsnoop, like interceptingWriter.
func TestInterceptingWriter_reimplementInterfaces(t *testing.T) {
	// combination 1/32
	{
		t.Log("http.ResponseWriter")
		inner := struct {
			http.ResponseWriter
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 2/32
	{
		t.Log("http.ResponseWriter, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 3/32
	{
		t.Log("http.ResponseWriter, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 4/32
	{
		t.Log("http.ResponseWriter, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 5/32
	{
		t.Log("http.ResponseWriter, http.Hijacker")
		inner := struct {
			http.ResponseWriter
			http.Hijacker
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 6/32
	{
		t.Log("http.ResponseWriter, http.Hijacker, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Hijacker
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 7/32
	{
		t.Log("http.ResponseWriter, http.Hijacker, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.Hijacker
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 8/32
	{
		t.Log("http.ResponseWriter, http.Hijacker, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Hijacker
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 9/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 10/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 11/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 12/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 13/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, http.Hijacker")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			http.Hijacker
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 14/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, http.Hijacker, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			http.Hijacker
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 15/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, http.Hijacker, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			http.Hijacker
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 16/32
	{
		t.Log("http.ResponseWriter, http.CloseNotifier, http.Hijacker, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.CloseNotifier
			http.Hijacker
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 17/32
	{
		t.Log("http.ResponseWriter, http.Flusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 18/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 19/32
	{
		t.Log("http.ResponseWriter, http.Flusher, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 20/32
	{
		t.Log("http.ResponseWriter, http.Flusher, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 21/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.Hijacker")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.Hijacker
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 22/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.Hijacker, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.Hijacker
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 23/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.Hijacker, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.Hijacker
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 24/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.Hijacker, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.Hijacker
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 25/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 26/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 27/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 28/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 29/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, http.Hijacker")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			http.Hijacker
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 30/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, http.Hijacker, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			http.Hijacker
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != false {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}

	// combination 31/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, http.Hijacker, io.ReaderFrom")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			http.Hijacker
			io.ReaderFrom
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != false {
			t.Error("unexpected interface")
		}

	}

	// combination 32/32
	{
		t.Log("http.ResponseWriter, http.Flusher, http.CloseNotifier, http.Hijacker, io.ReaderFrom, http.Pusher")
		inner := struct {
			http.ResponseWriter
			http.Flusher
			http.CloseNotifier
			http.Hijacker
			io.ReaderFrom
			http.Pusher
		}{}
		w := (&interceptingWriter{ResponseWriter: inner}).reimplementInterfaces()
		if _, ok := w.(http.ResponseWriter); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Flusher); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.CloseNotifier); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Hijacker); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(io.ReaderFrom); ok != true {
			t.Error("unexpected interface")
		}
		if _, ok := w.(http.Pusher); ok != true {
			t.Error("unexpected interface")
		}

	}
}
