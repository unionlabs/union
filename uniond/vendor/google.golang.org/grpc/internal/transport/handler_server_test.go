/*
 *
 * Copyright 2016 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package transport

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"sync"
	"testing"
	"time"

	epb "google.golang.org/genproto/googleapis/rpc/errdetails"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/mem"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/protoadapt"
	"google.golang.org/protobuf/types/known/durationpb"
)

func (s) TestHandlerTransport_NewServerHandlerTransport(t *testing.T) {
	type testCase struct {
		name        string
		req         *http.Request
		wantErr     string
		wantErrCode int
		modrw       func(http.ResponseWriter) http.ResponseWriter
		check       func(*serverHandlerTransport, *testCase) error
	}
	tests := []testCase{
		{
			name: "bad method",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "GET",
				Header:     http.Header{},
			},
			wantErr:     `invalid gRPC request method "GET"`,
			wantErrCode: http.StatusMethodNotAllowed,
		},
		{
			name: "bad content type",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": {"application/foo"},
				},
			},
			wantErr:     `invalid gRPC request content-type "application/foo"`,
			wantErrCode: http.StatusUnsupportedMediaType,
		},
		{
			name: "http/1.1",
			req: &http.Request{
				ProtoMajor: 1,
				ProtoMinor: 1,
				Method:     "POST",
				Header:     http.Header{"Content-Type": []string{"application/grpc"}},
			},
			wantErr:     "gRPC requires HTTP/2",
			wantErrCode: http.StatusHTTPVersionNotSupported,
		},
		{
			name: "not flusher",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": {"application/grpc"},
				},
			},
			modrw: func(w http.ResponseWriter) http.ResponseWriter {
				// Return w without its Flush method
				type onlyCloseNotifier interface {
					http.ResponseWriter
				}
				return struct{ onlyCloseNotifier }{w.(onlyCloseNotifier)}
			},
			wantErr:     "gRPC requires a ResponseWriter supporting http.Flusher",
			wantErrCode: http.StatusInternalServerError,
		},
		{
			name: "valid",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": {"application/grpc"},
				},
				URL: &url.URL{
					Path: "/service/foo.bar",
				},
			},
			check: func(t *serverHandlerTransport, tt *testCase) error {
				if t.req != tt.req {
					return fmt.Errorf("t.req = %p; want %p", t.req, tt.req)
				}
				if t.rw == nil {
					return errors.New("t.rw = nil; want non-nil")
				}
				return nil
			},
		},
		{
			name: "with timeout",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": []string{"application/grpc"},
					"Grpc-Timeout": {"200m"},
				},
				URL: &url.URL{
					Path: "/service/foo.bar",
				},
			},
			check: func(t *serverHandlerTransport, _ *testCase) error {
				if !t.timeoutSet {
					return errors.New("timeout not set")
				}
				if want := 200 * time.Millisecond; t.timeout != want {
					return fmt.Errorf("timeout = %v; want %v", t.timeout, want)
				}
				return nil
			},
		},
		{
			name: "with bad timeout",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": []string{"application/grpc"},
					"Grpc-Timeout": {"tomorrow"},
				},
				URL: &url.URL{
					Path: "/service/foo.bar",
				},
			},
			wantErr:     `rpc error: code = Internal desc = malformed grpc-timeout: transport: timeout unit is not recognized: "tomorrow"`,
			wantErrCode: http.StatusBadRequest,
		},
		{
			name: "with metadata",
			req: &http.Request{
				ProtoMajor: 2,
				Method:     "POST",
				Header: http.Header{
					"Content-Type": []string{"application/grpc"},
					"meta-foo":     {"foo-val"},
					"meta-bar":     {"bar-val1", "bar-val2"},
					"user-agent":   {"x/y a/b"},
				},
				URL: &url.URL{
					Path: "/service/foo.bar",
				},
			},
			check: func(ht *serverHandlerTransport, _ *testCase) error {
				want := metadata.MD{
					"meta-bar":     {"bar-val1", "bar-val2"},
					"user-agent":   {"x/y a/b"},
					"meta-foo":     {"foo-val"},
					"content-type": {"application/grpc"},
				}

				if !reflect.DeepEqual(ht.headerMD, want) {
					return fmt.Errorf("metadata = %#v; want %#v", ht.headerMD, want)
				}
				return nil
			},
		},
	}

	for _, tt := range tests {
		rrec := httptest.NewRecorder()
		rw := http.ResponseWriter(testHandlerResponseWriter{
			ResponseRecorder: rrec,
		})

		if tt.modrw != nil {
			rw = tt.modrw(rw)
		}
		got, gotErr := NewServerHandlerTransport(rw, tt.req, nil, mem.DefaultBufferPool())
		if (gotErr != nil) != (tt.wantErr != "") || (gotErr != nil && gotErr.Error() != tt.wantErr) {
			t.Errorf("%s: error = %q; want %q", tt.name, gotErr.Error(), tt.wantErr)
			continue
		}
		if tt.wantErrCode == 0 {
			tt.wantErrCode = http.StatusOK
		}
		if rrec.Code != tt.wantErrCode {
			t.Errorf("%s: code = %d; want %d", tt.name, rrec.Code, tt.wantErrCode)
			continue
		}
		if gotErr != nil {
			continue
		}
		if tt.check != nil {
			if err := tt.check(got.(*serverHandlerTransport), &tt); err != nil {
				t.Errorf("%s: %v", tt.name, err)
			}
		}
	}
}

type testHandlerResponseWriter struct {
	*httptest.ResponseRecorder
}

func (w testHandlerResponseWriter) Flush() {}

func newTestHandlerResponseWriter() http.ResponseWriter {
	return testHandlerResponseWriter{
		ResponseRecorder: httptest.NewRecorder(),
	}
}

type handleStreamTest struct {
	t     *testing.T
	bodyw *io.PipeWriter
	rw    testHandlerResponseWriter
	ht    *serverHandlerTransport
}

func newHandleStreamTest(t *testing.T) *handleStreamTest {
	bodyr, bodyw := io.Pipe()
	req := &http.Request{
		ProtoMajor: 2,
		Method:     "POST",
		Header: http.Header{
			"Content-Type": {"application/grpc"},
		},
		URL: &url.URL{
			Path: "/service/foo.bar",
		},
		Body: bodyr,
	}
	rw := newTestHandlerResponseWriter().(testHandlerResponseWriter)
	ht, err := NewServerHandlerTransport(rw, req, nil, mem.DefaultBufferPool())
	if err != nil {
		t.Fatal(err)
	}
	return &handleStreamTest{
		t:     t,
		bodyw: bodyw,
		ht:    ht.(*serverHandlerTransport),
		rw:    rw,
	}
}

func (s) TestHandlerTransport_HandleStreams(t *testing.T) {
	st := newHandleStreamTest(t)
	handleStream := func(s *Stream) {
		if want := "/service/foo.bar"; s.method != want {
			t.Errorf("stream method = %q; want %q", s.method, want)
		}

		if err := s.SetHeader(metadata.Pairs("custom-header", "Custom header value")); err != nil {
			t.Error(err)
		}

		if err := s.SetTrailer(metadata.Pairs("custom-trailer", "Custom trailer value")); err != nil {
			t.Error(err)
		}

		if err := s.SetSendCompress("gzip"); err != nil {
			t.Error(err)
		}

		md := metadata.Pairs("custom-header", "Another custom header value")
		if err := s.SendHeader(md); err != nil {
			t.Error(err)
		}
		delete(md, "custom-header")

		if err := s.SetHeader(metadata.Pairs("too-late", "Header value that should be ignored")); err == nil {
			t.Error("expected SetHeader call after SendHeader to fail")
		}

		if err := s.SendHeader(metadata.Pairs("too-late", "This header value should be ignored as well")); err == nil {
			t.Error("expected second SendHeader call to fail")
		}

		if err := s.SetSendCompress("snappy"); err == nil {
			t.Error("expected second SetSendCompress call to fail")
		}

		st.bodyw.Close() // no body
		st.ht.WriteStatus(s, status.New(codes.OK, ""))
	}
	st.ht.HandleStreams(
		context.Background(), func(s *Stream) { go handleStream(s) },
	)
	wantHeader := http.Header{
		"Date":          nil,
		"Content-Type":  {"application/grpc"},
		"Trailer":       {"Grpc-Status", "Grpc-Message", "Grpc-Status-Details-Bin"},
		"Custom-Header": {"Custom header value", "Another custom header value"},
		"Grpc-Encoding": {"gzip"},
	}
	wantTrailer := http.Header{
		"Grpc-Status":    {"0"},
		"Custom-Trailer": {"Custom trailer value"},
	}
	checkHeaderAndTrailer(t, st.rw, wantHeader, wantTrailer)
}

// Tests that codes.Unimplemented will close the body, per comment in handler_server.go.
func (s) TestHandlerTransport_HandleStreams_Unimplemented(t *testing.T) {
	handleStreamCloseBodyTest(t, codes.Unimplemented, "thingy is unimplemented")
}

// Tests that codes.InvalidArgument will close the body, per comment in handler_server.go.
func (s) TestHandlerTransport_HandleStreams_InvalidArgument(t *testing.T) {
	handleStreamCloseBodyTest(t, codes.InvalidArgument, "bad arg")
}

func handleStreamCloseBodyTest(t *testing.T, statusCode codes.Code, msg string) {
	st := newHandleStreamTest(t)

	handleStream := func(s *Stream) {
		st.ht.WriteStatus(s, status.New(statusCode, msg))
	}
	st.ht.HandleStreams(
		context.Background(), func(s *Stream) { go handleStream(s) },
	)
	wantHeader := http.Header{
		"Date":         nil,
		"Content-Type": {"application/grpc"},
		"Trailer":      {"Grpc-Status", "Grpc-Message", "Grpc-Status-Details-Bin"},
	}
	wantTrailer := http.Header{
		"Grpc-Status":  {fmt.Sprint(uint32(statusCode))},
		"Grpc-Message": {encodeGrpcMessage(msg)},
	}
	checkHeaderAndTrailer(t, st.rw, wantHeader, wantTrailer)
}

func (s) TestHandlerTransport_HandleStreams_Timeout(t *testing.T) {
	bodyr, bodyw := io.Pipe()
	req := &http.Request{
		ProtoMajor: 2,
		Method:     "POST",
		Header: http.Header{
			"Content-Type": {"application/grpc"},
			"Grpc-Timeout": {"200m"},
		},
		URL: &url.URL{
			Path: "/service/foo.bar",
		},
		Body: bodyr,
	}
	rw := newTestHandlerResponseWriter().(testHandlerResponseWriter)
	ht, err := NewServerHandlerTransport(rw, req, nil, mem.DefaultBufferPool())
	if err != nil {
		t.Fatal(err)
	}
	runStream := func(s *Stream) {
		defer bodyw.Close()
		select {
		case <-s.ctx.Done():
		case <-time.After(5 * time.Second):
			t.Errorf("timeout waiting for ctx.Done")
			return
		}
		err := s.ctx.Err()
		if err != context.DeadlineExceeded {
			t.Errorf("ctx.Err = %v; want %v", err, context.DeadlineExceeded)
			return
		}
		ht.WriteStatus(s, status.New(codes.DeadlineExceeded, "too slow"))
	}
	ht.HandleStreams(
		context.Background(), func(s *Stream) { go runStream(s) },
	)
	wantHeader := http.Header{
		"Date":         nil,
		"Content-Type": {"application/grpc"},
		"Trailer":      {"Grpc-Status", "Grpc-Message", "Grpc-Status-Details-Bin"},
	}
	wantTrailer := http.Header{
		"Grpc-Status":  {"4"},
		"Grpc-Message": {encodeGrpcMessage("too slow")},
	}
	checkHeaderAndTrailer(t, rw, wantHeader, wantTrailer)
}

// TestHandlerTransport_HandleStreams_MultiWriteStatus ensures that
// concurrent "WriteStatus"s do not panic writing to closed "writes" channel.
func (s) TestHandlerTransport_HandleStreams_MultiWriteStatus(t *testing.T) {
	testHandlerTransportHandleStreams(t, func(st *handleStreamTest, s *Stream) {
		if want := "/service/foo.bar"; s.method != want {
			t.Errorf("stream method = %q; want %q", s.method, want)
		}
		st.bodyw.Close() // no body

		var wg sync.WaitGroup
		wg.Add(5)
		for i := 0; i < 5; i++ {
			go func() {
				defer wg.Done()
				st.ht.WriteStatus(s, status.New(codes.OK, ""))
			}()
		}
		wg.Wait()
	})
}

// TestHandlerTransport_HandleStreams_WriteStatusWrite ensures that "Write"
// following "WriteStatus" does not panic writing to closed "writes" channel.
func (s) TestHandlerTransport_HandleStreams_WriteStatusWrite(t *testing.T) {
	testHandlerTransportHandleStreams(t, func(st *handleStreamTest, s *Stream) {
		if want := "/service/foo.bar"; s.method != want {
			t.Errorf("stream method = %q; want %q", s.method, want)
		}
		st.bodyw.Close() // no body

		st.ht.WriteStatus(s, status.New(codes.OK, ""))
		st.ht.Write(s, []byte("hdr"), newBufferSlice([]byte("data")), &Options{})
	})
}

func testHandlerTransportHandleStreams(t *testing.T, handleStream func(st *handleStreamTest, s *Stream)) {
	st := newHandleStreamTest(t)
	st.ht.HandleStreams(
		context.Background(), func(s *Stream) { go handleStream(st, s) },
	)
}

func (s) TestHandlerTransport_HandleStreams_ErrDetails(t *testing.T) {
	errDetails := []protoadapt.MessageV1{
		&epb.RetryInfo{
			RetryDelay: &durationpb.Duration{Seconds: 60},
		},
		&epb.ResourceInfo{
			ResourceType: "foo bar",
			ResourceName: "service.foo.bar",
			Owner:        "User",
		},
	}

	statusCode := codes.ResourceExhausted
	msg := "you are being throttled"
	st, err := status.New(statusCode, msg).WithDetails(errDetails...)
	if err != nil {
		t.Fatal(err)
	}

	stBytes, err := proto.Marshal(st.Proto())
	if err != nil {
		t.Fatal(err)
	}

	hst := newHandleStreamTest(t)
	handleStream := func(s *Stream) {
		hst.ht.WriteStatus(s, st)
	}
	hst.ht.HandleStreams(
		context.Background(), func(s *Stream) { go handleStream(s) },
	)
	wantHeader := http.Header{
		"Date":         nil,
		"Content-Type": {"application/grpc"},
		"Trailer":      {"Grpc-Status", "Grpc-Message", "Grpc-Status-Details-Bin"},
	}
	wantTrailer := http.Header{
		"Grpc-Status":             {fmt.Sprint(uint32(statusCode))},
		"Grpc-Message":            {encodeGrpcMessage(msg)},
		"Grpc-Status-Details-Bin": {encodeBinHeader(stBytes)},
	}

	checkHeaderAndTrailer(t, hst.rw, wantHeader, wantTrailer)
}

// TestHandlerTransport_Drain verifies that Drain() is not implemented
// by `serverHandlerTransport`.
func (s) TestHandlerTransport_Drain(t *testing.T) {
	defer func() { recover() }()
	st := newHandleStreamTest(t)
	st.ht.Drain("whatever")
	t.Errorf("serverHandlerTransport.Drain() should have panicked")
}

// checkHeaderAndTrailer checks that the resulting header and trailer matches the expectation.
func checkHeaderAndTrailer(t *testing.T, rw testHandlerResponseWriter, wantHeader, wantTrailer http.Header) {
	// For trailer-only responses, the trailer values might be reported as part of the Header. They will however
	// be present in Trailer in either case. Hence, normalize the header by removing all trailer values.
	actualHeader := rw.Result().Header.Clone()
	for _, trailerKey := range actualHeader["Trailer"] {
		actualHeader.Del(trailerKey)
	}

	if !reflect.DeepEqual(actualHeader, wantHeader) {
		t.Errorf("Header mismatch.\n got: %#v\n want: %#v", actualHeader, wantHeader)
	}
	if actualTrailer := rw.Result().Trailer; !reflect.DeepEqual(actualTrailer, wantTrailer) {
		t.Errorf("Trailer mismatch.\n got: %#v\n want: %#v", actualTrailer, wantTrailer)
	}
}
