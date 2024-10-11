// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/rand"
	"errors"
	"fmt"
	"io"
	"strings"
	"testing"
)

func TestStreamWriteBlockedByOutputBuffer(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
		const writeBufferSize = 4
		tc := newTestConn(t, clientSide, permissiveTransportParameters, func(c *Config) {
			c.MaxStreamWriteBufferSize = writeBufferSize
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		s := newLocalStream(t, tc, styp)

		// Non-blocking write.
		n, err := s.Write(want)
		if n != writeBufferSize || err != context.Canceled {
			t.Fatalf("s.Write() = %v, %v; want %v, context.Canceled", n, err, writeBufferSize)
		}
		s.Flush()
		tc.wantFrame("first write buffer of data sent",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				data: want[:writeBufferSize],
			})
		off := int64(writeBufferSize)

		// Blocking write, which must wait for buffer space.
		w := runAsync(tc, func(ctx context.Context) (int, error) {
			s.SetWriteContext(ctx)
			n, err := s.Write(want[writeBufferSize:])
			s.Flush()
			return n, err
		})
		tc.wantIdle("write buffer is full, no more data can be sent")

		// The peer's ack of the STREAM frame allows progress.
		tc.writeAckForAll()
		tc.wantFrame("second write buffer of data sent",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  off,
				data: want[off:][:writeBufferSize],
			})
		off += writeBufferSize
		tc.wantIdle("write buffer is full, no more data can be sent")

		// The peer's ack of the second STREAM frame allows sending the remaining data.
		tc.writeAckForAll()
		tc.wantFrame("remaining data sent",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  off,
				data: want[off:],
			})

		if n, err := w.result(); n != len(want)-writeBufferSize || err != nil {
			t.Fatalf("s.Write() = %v, %v; want %v, nil",
				len(want)-writeBufferSize, err, writeBufferSize)
		}
	})
}

func TestStreamWriteBlockedByStreamFlowControl(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		ctx := canceledContext()
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
		tc := newTestConn(t, clientSide, func(p *transportParameters) {
			p.initialMaxStreamsBidi = 100
			p.initialMaxStreamsUni = 100
			p.initialMaxData = 1 << 20
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		s, err := tc.conn.newLocalStream(ctx, styp)
		if err != nil {
			t.Fatal(err)
		}

		// Data is written to the stream output buffer, but we have no flow control.
		_, err = s.Write(want[:1])
		if err != nil {
			t.Fatalf("write with available output buffer: unexpected error: %v", err)
		}
		s.Flush()
		tc.wantFrame("write blocked by flow control triggers a STREAM_DATA_BLOCKED frame",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 0,
			})

		// Write more data.
		_, err = s.Write(want[1:])
		if err != nil {
			t.Fatalf("write with available output buffer: unexpected error: %v", err)
		}
		s.Flush()
		tc.wantIdle("adding more blocked data does not trigger another STREAM_DATA_BLOCKED")

		// Provide some flow control window.
		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 4,
		})
		tc.wantFrame("stream window extended, but still more data to write",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 4,
			})
		tc.wantFrame("stream window extended to 4, expect blocked write to progress",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				data: want[:4],
			})

		// Provide more flow control window.
		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: int64(len(want)),
		})
		tc.wantFrame("stream window extended further, expect blocked write to finish",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  4,
				data: want[4:],
			})
	})
}

func TestStreamIgnoresMaxStreamDataReduction(t *testing.T) {
	// "A sender MUST ignore any MAX_STREAM_DATA [...] frames that
	// do not increase flow control limits."
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.1-9
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		ctx := canceledContext()
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
		tc := newTestConn(t, clientSide, func(p *transportParameters) {
			if styp == uniStream {
				p.initialMaxStreamsUni = 1
				p.initialMaxStreamDataUni = 4
			} else {
				p.initialMaxStreamsBidi = 1
				p.initialMaxStreamDataBidiRemote = 4
			}
			p.initialMaxData = 1 << 20
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		tc.ignoreFrame(frameTypeStreamDataBlocked)

		// Write [0,1).
		s, err := tc.conn.newLocalStream(ctx, styp)
		if err != nil {
			t.Fatal(err)
		}
		s.Write(want[:1])
		s.Flush()
		tc.wantFrame("sent data (1 byte) fits within flow control limit",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: want[:1],
			})

		// MAX_STREAM_DATA tries to decrease limit, and is ignored.
		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 2,
		})

		// Write [1,4).
		s.Write(want[1:])
		tc.wantFrame("stream limit is 4 bytes, ignoring decrease in MAX_STREAM_DATA",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  1,
				data: want[1:4],
			})

		// MAX_STREAM_DATA increases limit.
		// Second MAX_STREAM_DATA decreases it, and is ignored.
		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 8,
		})
		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 6,
		})

		// Write [1,4).
		s.Write(want[4:])
		tc.wantFrame("stream limit is 8 bytes, ignoring decrease in MAX_STREAM_DATA",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  4,
				data: want[4:8],
			})
	})
}

func TestStreamWriteBlockedByWriteBufferLimit(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
		const maxWriteBuffer = 4
		tc := newTestConn(t, clientSide, func(p *transportParameters) {
			p.initialMaxStreamsBidi = 100
			p.initialMaxStreamsUni = 100
			p.initialMaxData = 1 << 20
			p.initialMaxStreamDataBidiRemote = 1 << 20
			p.initialMaxStreamDataUni = 1 << 20
		}, func(c *Config) {
			c.MaxStreamWriteBufferSize = maxWriteBuffer
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		// Write more data than StreamWriteBufferSize.
		// The peer has given us plenty of flow control,
		// so we're just blocked by our local limit.
		s := newLocalStream(t, tc, styp)
		w := runAsync(tc, func(ctx context.Context) (int, error) {
			s.SetWriteContext(ctx)
			return s.Write(want)
		})
		tc.wantFrame("stream write should send as much data as write buffer allows",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: want[:maxWriteBuffer],
			})
		tc.wantIdle("no STREAM_DATA_BLOCKED, we're blocked locally not by flow control")

		// ACK for previously-sent data allows making more progress.
		tc.writeAckForAll()
		tc.wantFrame("ACK for previous data allows making progress",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  maxWriteBuffer,
				data: want[maxWriteBuffer:][:maxWriteBuffer],
			})

		// Cancel the write with data left to send.
		w.cancel()
		n, err := w.result()
		if n != 2*maxWriteBuffer || err == nil {
			t.Fatalf("Write() = %v, %v; want %v bytes, error", n, err, 2*maxWriteBuffer)
		}
	})
}

func TestStreamReceive(t *testing.T) {
	// "Endpoints MUST be able to deliver stream data to an application as
	// an ordered byte stream."
	// https://www.rfc-editor.org/rfc/rfc9000#section-2.2-2
	want := make([]byte, 5000)
	for i := range want {
		want[i] = byte(i)
	}
	type frame struct {
		start   int64
		end     int64
		fin     bool
		want    int
		wantEOF bool
	}
	for _, test := range []struct {
		name   string
		frames []frame
	}{{
		name: "linear",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}, {
			start:   2000,
			end:     3000,
			want:    3000,
			fin:     true,
			wantEOF: true,
		}},
	}, {
		name: "out of order",
		frames: []frame{{
			start: 1000,
			end:   2000,
		}, {
			start: 2000,
			end:   3000,
		}, {
			start: 0,
			end:   1000,
			want:  3000,
		}},
	}, {
		name: "resent",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}, {
			start: 0,
			end:   1000,
			want:  2000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}},
	}, {
		name: "overlapping",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 3000,
			end:   4000,
			want:  1000,
		}, {
			start: 2000,
			end:   3000,
			want:  1000,
		}, {
			start: 1000,
			end:   3000,
			want:  4000,
		}},
	}, {
		name: "early eof",
		frames: []frame{{
			start: 3000,
			end:   3000,
			fin:   true,
			want:  0,
		}, {
			start: 1000,
			end:   2000,
			want:  0,
		}, {
			start: 0,
			end:   1000,
			want:  2000,
		}, {
			start:   2000,
			end:     3000,
			want:    3000,
			wantEOF: true,
		}},
	}, {
		name: "empty eof",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start:   1000,
			end:     1000,
			fin:     true,
			want:    1000,
			wantEOF: true,
		}},
	}} {
		testStreamTypes(t, test.name, func(t *testing.T, styp streamType) {
			tc := newTestConn(t, serverSide)
			tc.handshake()
			sid := newStreamID(clientSide, styp, 0)
			var s *Stream
			got := make([]byte, len(want))
			var total int
			for _, f := range test.frames {
				t.Logf("receive [%v,%v)", f.start, f.end)
				tc.writeFrames(packetType1RTT, debugFrameStream{
					id:   sid,
					off:  f.start,
					data: want[f.start:f.end],
					fin:  f.fin,
				})
				if s == nil {
					s = tc.acceptStream()
				}
				for {
					n, err := s.Read(got[total:])
					t.Logf("s.Read() = %v, %v", n, err)
					total += n
					if f.wantEOF && err != io.EOF {
						t.Fatalf("Read() error = %v; want io.EOF", err)
					}
					if !f.wantEOF && err == io.EOF {
						t.Fatalf("Read() error = io.EOF, want something else")
					}
					if err != nil {
						break
					}
				}
				if total != f.want {
					t.Fatalf("total bytes read = %v, want %v", total, f.want)
				}
				for i := 0; i < total; i++ {
					if got[i] != want[i] {
						t.Fatalf("byte %v differs: got %v, want %v", i, got[i], want[i])
					}
				}
			}
		})
	}

}

func TestStreamReceiveExtendsStreamWindow(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		const maxWindowSize = 20
		ctx := canceledContext()
		tc := newTestConn(t, serverSide, func(c *Config) {
			c.MaxStreamReadBufferSize = maxWindowSize
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		sid := newStreamID(clientSide, styp, 0)
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  0,
			data: make([]byte, maxWindowSize),
		})
		s, err := tc.conn.AcceptStream(ctx)
		if err != nil {
			t.Fatalf("AcceptStream: %v", err)
		}
		tc.wantIdle("stream window is not extended before data is read")
		buf := make([]byte, maxWindowSize+1)
		if n, err := s.Read(buf); n != maxWindowSize || err != nil {
			t.Fatalf("s.Read() = %v, %v; want %v, nil", n, err, maxWindowSize)
		}
		tc.wantFrame("stream window is extended after reading data",
			packetType1RTT, debugFrameMaxStreamData{
				id:  sid,
				max: maxWindowSize * 2,
			})
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  maxWindowSize,
			data: make([]byte, maxWindowSize),
			fin:  true,
		})
		if n, err := s.Read(buf); n != maxWindowSize || err != io.EOF {
			t.Fatalf("s.Read() = %v, %v; want %v, io.EOF", n, err, maxWindowSize)
		}
		tc.wantIdle("stream window is not extended after FIN")
	})
}

func TestStreamReceiveViolatesStreamDataLimit(t *testing.T) {
	// "A receiver MUST close the connection with an error of type FLOW_CONTROL_ERROR if
	// the sender violates the advertised [...] stream data limits [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.1-8
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		const maxStreamData = 10
		for _, test := range []struct {
			off  int64
			size int64
		}{{
			off:  maxStreamData,
			size: 1,
		}, {
			off:  0,
			size: maxStreamData + 1,
		}, {
			off:  maxStreamData - 1,
			size: 2,
		}} {
			tc := newTestConn(t, serverSide, func(c *Config) {
				c.MaxStreamReadBufferSize = maxStreamData
			})
			tc.handshake()
			tc.ignoreFrame(frameTypeAck)
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id:   newStreamID(clientSide, styp, 0),
				off:  test.off,
				data: make([]byte, test.size),
			})
			tc.wantFrame(
				fmt.Sprintf("data [%v,%v) violates stream data limit and closes connection",
					test.off, test.off+test.size),
				packetType1RTT, debugFrameConnectionCloseTransport{
					code: errFlowControl,
				},
			)
		}
	})
}

func TestStreamReceiveDuplicateDataDoesNotViolateLimits(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		const maxData = 10
		tc := newTestConn(t, serverSide, func(c *Config) {
			// TODO: Add connection-level maximum data here as well.
			c.MaxStreamReadBufferSize = maxData
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		for i := 0; i < 3; i++ {
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id:   newStreamID(clientSide, styp, 0),
				off:  0,
				data: make([]byte, maxData),
			})
			tc.wantIdle(fmt.Sprintf("conn sends no frames after receiving data frame %v", i))
		}
	})
}

func TestStreamReceiveEmptyEOF(t *testing.T) {
	// A stream receives some data, we read a byte of that data
	// (causing the rest to be pulled into the s.inbuf buffer),
	// and then we receive a FIN with no additional data.
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndRemoteStream(t, serverSide, styp, permissiveTransportParameters)
		want := []byte{1, 2, 3}
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			data: want,
		})
		if got, err := s.ReadByte(); got != want[0] || err != nil {
			t.Fatalf("s.ReadByte() = %v, %v; want %v, nil", got, err, want[0])
		}

		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:  s.id,
			off: 3,
			fin: true,
		})
		if got, err := io.ReadAll(s); !bytes.Equal(got, want[1:]) || err != nil {
			t.Fatalf("io.ReadAll(s) = {%x}, %v; want {%x}, nil", got, err, want[1:])
		}
	})
}

func finalSizeTest(t *testing.T, wantErr transportError, f func(tc *testConn, sid streamID) (finalSize int64), opts ...any) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		for _, test := range []struct {
			name       string
			finalFrame func(tc *testConn, sid streamID, finalSize int64)
		}{{
			name: "FIN",
			finalFrame: func(tc *testConn, sid streamID, finalSize int64) {
				tc.writeFrames(packetType1RTT, debugFrameStream{
					id:  sid,
					off: finalSize,
					fin: true,
				})
			},
		}, {
			name: "RESET_STREAM",
			finalFrame: func(tc *testConn, sid streamID, finalSize int64) {
				tc.writeFrames(packetType1RTT, debugFrameResetStream{
					id:        sid,
					finalSize: finalSize,
				})
			},
		}} {
			t.Run(test.name, func(t *testing.T) {
				tc := newTestConn(t, serverSide, opts...)
				tc.handshake()
				sid := newStreamID(clientSide, styp, 0)
				finalSize := f(tc, sid)
				test.finalFrame(tc, sid, finalSize)
				tc.wantFrame("change in final size of stream is an error",
					packetType1RTT, debugFrameConnectionCloseTransport{
						code: wantErr,
					},
				)
			})
		}
	})
}

func TestStreamFinalSizeChangedAfterFin(t *testing.T) {
	// "If a RESET_STREAM or STREAM frame is received indicating a change
	// in the final size for the stream, an endpoint SHOULD respond with
	// an error of type FINAL_SIZE_ERROR [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.5-5
	finalSizeTest(t, errFinalSize, func(tc *testConn, sid streamID) (finalSize int64) {
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:  sid,
			off: 10,
			fin: true,
		})
		return 9
	})
}

func TestStreamFinalSizeBeforePreviousData(t *testing.T) {
	finalSizeTest(t, errFinalSize, func(tc *testConn, sid streamID) (finalSize int64) {
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  10,
			data: []byte{0},
		})
		return 9
	})
}

func TestStreamFinalSizePastMaxStreamData(t *testing.T) {
	finalSizeTest(t, errFlowControl, func(tc *testConn, sid streamID) (finalSize int64) {
		return 11
	}, func(c *Config) {
		c.MaxStreamReadBufferSize = 10
	})
}

func TestStreamDataBeyondFinalSize(t *testing.T) {
	// "A receiver SHOULD treat receipt of data at or beyond
	// the final size as an error of type FINAL_SIZE_ERROR [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.5-5
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide)
		tc.handshake()
		sid := newStreamID(clientSide, styp, 0)

		const write1size = 4
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  0,
			data: make([]byte, 16),
			fin:  true,
		})
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  16,
			data: []byte{0},
		})
		tc.wantFrame("received data past final size of stream",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errFinalSize,
			},
		)
	})
}

func TestStreamReceiveUnblocksReader(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide)
		tc.handshake()
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
		sid := newStreamID(clientSide, styp, 0)

		// AcceptStream blocks until a STREAM frame is received.
		accept := runAsync(tc, func(ctx context.Context) (*Stream, error) {
			return tc.conn.AcceptStream(ctx)
		})
		const write1size = 4
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  0,
			data: want[:write1size],
		})
		s, err := accept.result()
		if err != nil {
			t.Fatalf("AcceptStream() = %v", err)
		}

		// Read succeeds immediately, since we already have data.
		got := make([]byte, len(want))
		read := runAsync(tc, func(ctx context.Context) (int, error) {
			return s.Read(got)
		})
		if n, err := read.result(); n != write1size || err != nil {
			t.Fatalf("Read = %v, %v; want %v, nil", n, err, write1size)
		}

		// Read blocks waiting for more data.
		read = runAsync(tc, func(ctx context.Context) (int, error) {
			s.SetReadContext(ctx)
			return s.Read(got[write1size:])
		})
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   sid,
			off:  write1size,
			data: want[write1size:],
			fin:  true,
		})
		if n, err := read.result(); n != len(want)-write1size || err != io.EOF {
			t.Fatalf("Read = %v, %v; want %v, io.EOF", n, err, len(want)-write1size)
		}
		if !bytes.Equal(got, want) {
			t.Fatalf("read bytes %x, want %x", got, want)
		}
	})
}

// testStreamSendFrameInvalidState calls the test func with a stream ID for:
//
//   - a remote bidirectional stream that the peer has not created
//   - a remote unidirectional stream
//
// It then sends the returned frame (STREAM, STREAM_DATA_BLOCKED, etc.)
// to the conn and expects a STREAM_STATE_ERROR.
func testStreamSendFrameInvalidState(t *testing.T, f func(sid streamID) debugFrame) {
	testSides(t, "stream_not_created", func(t *testing.T, side connSide) {
		tc := newTestConn(t, side, permissiveTransportParameters)
		tc.handshake()
		tc.writeFrames(packetType1RTT, f(newStreamID(side, bidiStream, 0)))
		tc.wantFrame("frame for local stream which has not been created",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamState,
			})
	})
	testSides(t, "uni_stream", func(t *testing.T, side connSide) {
		ctx := canceledContext()
		tc := newTestConn(t, side, permissiveTransportParameters)
		tc.handshake()
		sid := newStreamID(side, uniStream, 0)
		s, err := tc.conn.NewSendOnlyStream(ctx)
		if err != nil {
			t.Fatal(err)
		}
		s.Flush() // open the stream
		tc.wantFrame("new stream is opened",
			packetType1RTT, debugFrameStream{
				id:   sid,
				data: []byte{},
			})
		tc.writeFrames(packetType1RTT, f(sid))
		tc.wantFrame("send-oriented frame for send-only stream",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamState,
			})
	})
}

func TestStreamResetStreamInvalidState(t *testing.T) {
	// "An endpoint that receives a RESET_STREAM frame for a send-only
	// stream MUST terminate the connection with error STREAM_STATE_ERROR."
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.4-3
	testStreamSendFrameInvalidState(t, func(sid streamID) debugFrame {
		return debugFrameResetStream{
			id:        sid,
			code:      0,
			finalSize: 0,
		}
	})
}

func TestStreamStreamFrameInvalidState(t *testing.T) {
	// "An endpoint MUST terminate the connection with error STREAM_STATE_ERROR
	// if it receives a STREAM frame for a locally initiated stream
	// that has not yet been created, or for a send-only stream."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.8-3
	testStreamSendFrameInvalidState(t, func(sid streamID) debugFrame {
		return debugFrameStream{
			id: sid,
		}
	})
}

func TestStreamDataBlockedInvalidState(t *testing.T) {
	// "An endpoint MUST terminate the connection with error STREAM_STATE_ERROR
	// if it receives a STREAM frame for a locally initiated stream
	// that has not yet been created, or for a send-only stream."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.8-3
	testStreamSendFrameInvalidState(t, func(sid streamID) debugFrame {
		return debugFrameStream{
			id: sid,
		}
	})
}

// testStreamReceiveFrameInvalidState calls the test func with a stream ID for:
//
//   - a remote bidirectional stream that the peer has not created
//   - a local unidirectional stream
//
// It then sends the returned frame (MAX_STREAM_DATA, STOP_SENDING, etc.)
// to the conn and expects a STREAM_STATE_ERROR.
func testStreamReceiveFrameInvalidState(t *testing.T, f func(sid streamID) debugFrame) {
	testSides(t, "stream_not_created", func(t *testing.T, side connSide) {
		tc := newTestConn(t, side)
		tc.handshake()
		tc.writeFrames(packetType1RTT, f(newStreamID(side, bidiStream, 0)))
		tc.wantFrame("frame for local stream which has not been created",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamState,
			})
	})
	testSides(t, "uni_stream", func(t *testing.T, side connSide) {
		tc := newTestConn(t, side)
		tc.handshake()
		tc.writeFrames(packetType1RTT, f(newStreamID(side.peer(), uniStream, 0)))
		tc.wantFrame("receive-oriented frame for receive-only stream",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamState,
			})
	})
}

func TestStreamStopSendingInvalidState(t *testing.T) {
	// "Receiving a STOP_SENDING frame for a locally initiated stream
	// that has not yet been created MUST be treated as a connection error
	// of type STREAM_STATE_ERROR. An endpoint that receives a STOP_SENDING
	// frame for a receive-only stream MUST terminate the connection with
	// error STREAM_STATE_ERROR."
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.5-2
	testStreamReceiveFrameInvalidState(t, func(sid streamID) debugFrame {
		return debugFrameStopSending{
			id: sid,
		}
	})
}

func TestStreamMaxStreamDataInvalidState(t *testing.T) {
	// "Receiving a MAX_STREAM_DATA frame for a locally initiated stream
	// that has not yet been created MUST be treated as a connection error
	// of type STREAM_STATE_ERROR. An endpoint that receives a MAX_STREAM_DATA
	// frame for a receive-only stream MUST terminate the connection
	// with error STREAM_STATE_ERROR."
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.10-2
	testStreamReceiveFrameInvalidState(t, func(sid streamID) debugFrame {
		return debugFrameMaxStreamData{
			id:  sid,
			max: 1000,
		}
	})
}

func TestStreamOffsetTooLarge(t *testing.T) {
	// "Receipt of a frame that exceeds [2^62-1] MUST be treated as a
	// connection error of type FRAME_ENCODING_ERROR or FLOW_CONTROL_ERROR."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.8-9
	tc := newTestConn(t, serverSide)
	tc.handshake()

	tc.writeFrames(packetType1RTT,
		debugFrameStream{
			id:   newStreamID(clientSide, bidiStream, 0),
			off:  1<<62 - 1,
			data: []byte{0},
		})
	got, _ := tc.readFrame()
	want1 := debugFrameConnectionCloseTransport{code: errFrameEncoding}
	want2 := debugFrameConnectionCloseTransport{code: errFlowControl}
	if !frameEqual(got, want1) && !frameEqual(got, want2) {
		t.Fatalf("STREAM offset exceeds 2^62-1\ngot:  %v\nwant: %v\n  or: %v", got, want1, want2)
	}
}

func TestStreamReadFromWriteOnlyStream(t *testing.T) {
	_, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters)
	buf := make([]byte, 10)
	wantErr := "read from write-only stream"
	if n, err := s.Read(buf); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Read() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamWriteToReadOnlyStream(t *testing.T) {
	_, s := newTestConnAndRemoteStream(t, serverSide, uniStream)
	buf := make([]byte, 10)
	wantErr := "write to read-only stream"
	if n, err := s.Write(buf); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Write() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamReadFromClosedStream(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, bidiStream, permissiveTransportParameters)
	s.CloseRead()
	tc.wantFrame("CloseRead sends a STOP_SENDING frame",
		packetType1RTT, debugFrameStopSending{
			id: s.id,
		})
	wantErr := "read from closed stream"
	if n, err := s.Read(make([]byte, 16)); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Read() = %v, %v; want error %q", n, err, wantErr)
	}
	// Data which shows up after STOP_SENDING is discarded.
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		data: []byte{1, 2, 3},
		fin:  true,
	})
	if n, err := s.Read(make([]byte, 16)); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Read() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamCloseReadWithAllDataReceived(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, bidiStream, permissiveTransportParameters)
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		data: []byte{1, 2, 3},
		fin:  true,
	})
	s.CloseRead()
	tc.wantIdle("CloseRead in Data Recvd state doesn't need to send STOP_SENDING")
	// We had all the data for the stream, but CloseRead discarded it.
	wantErr := "read from closed stream"
	if n, err := s.Read(make([]byte, 16)); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Read() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamWriteToClosedStream(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, bidiStream, permissiveTransportParameters)
	s.CloseWrite()
	tc.wantFrame("stream is opened after being closed",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  0,
			fin:  true,
			data: []byte{},
		})
	wantErr := "write to closed stream"
	if n, err := s.Write([]byte{}); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Write() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamResetBlockedStream(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, bidiStream, permissiveTransportParameters,
		func(c *Config) {
			c.MaxStreamWriteBufferSize = 4
		})
	tc.ignoreFrame(frameTypeStreamDataBlocked)
	writing := runAsync(tc, func(ctx context.Context) (int, error) {
		s.SetWriteContext(ctx)
		return s.Write([]byte{0, 1, 2, 3, 4, 5, 6, 7})
	})
	tc.wantFrame("stream writes data until write buffer fills",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  0,
			data: []byte{0, 1, 2, 3},
		})
	s.Reset(42)
	tc.wantFrame("stream is reset",
		packetType1RTT, debugFrameResetStream{
			id:        s.id,
			code:      42,
			finalSize: 4,
		})
	wantErr := "write to reset stream"
	if n, err := writing.result(); n != 4 || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Write() interrupted by Reset: %v, %q; want 4, %q", n, err, wantErr)
	}
	tc.writeAckForAll()
	tc.wantIdle("buffer space is available, but stream has been reset")
	s.Reset(100)
	tc.wantIdle("resetting stream a second time has no effect")
	if n, err := s.Write([]byte{}); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("s.Write() = %v, %v; want error %q", n, err, wantErr)
	}
}

func TestStreamWriteMoreThanOnePacketOfData(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, func(p *transportParameters) {
		p.initialMaxStreamsUni = 1
		p.initialMaxData = 1 << 20
		p.initialMaxStreamDataUni = 1 << 20
	})
	want := make([]byte, 4096)
	rand.Read(want) // doesn't need to be crypto/rand, but non-deprecated and harmless
	w := runAsync(tc, func(ctx context.Context) (int, error) {
		n, err := s.Write(want)
		s.Flush()
		return n, err
	})
	got := make([]byte, 0, len(want))
	for {
		f, _ := tc.readFrame()
		if f == nil {
			break
		}
		sf, ok := f.(debugFrameStream)
		if !ok {
			t.Fatalf("unexpected frame: %v", sf)
		}
		if len(got) != int(sf.off) {
			t.Fatalf("got frame: %v\nwant offset %v", sf, len(got))
		}
		got = append(got, sf.data...)
	}
	if n, err := w.result(); n != len(want) || err != nil {
		t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(want))
	}
	if !bytes.Equal(got, want) {
		t.Fatalf("mismatch in received stream data")
	}
}

func TestStreamCloseWaitsForAcks(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters)
	data := make([]byte, 100)
	s.Write(data)
	s.Flush()
	tc.wantFrame("conn sends data for the stream",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data,
		})
	if err := s.Close(); err != context.Canceled {
		t.Fatalf("s.Close() = %v, want context.Canceled (data not acked yet)", err)
	}
	tc.wantFrame("conn sends FIN for closed stream",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  int64(len(data)),
			fin:  true,
			data: []byte{},
		})
	closing := runAsync(tc, func(ctx context.Context) (struct{}, error) {
		s.SetWriteContext(ctx)
		return struct{}{}, s.Close()
	})
	if _, err := closing.result(); err != errNotDone {
		t.Fatalf("s.Close() = %v, want it to block waiting for acks", err)
	}
	tc.writeAckForAll()
	if _, err := closing.result(); err != nil {
		t.Fatalf("s.Close() = %v, want nil (all data acked)", err)
	}
}

func TestStreamCloseReadOnly(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, permissiveTransportParameters)
	if err := s.Close(); err != nil {
		t.Errorf("s.Close() = %v, want nil", err)
	}
	tc.wantFrame("closed stream sends STOP_SENDING",
		packetType1RTT, debugFrameStopSending{
			id: s.id,
		})
}

func TestStreamCloseUnblocked(t *testing.T) {
	for _, test := range []struct {
		name    string
		unblock func(tc *testConn, s *Stream)
		success bool
	}{{
		name: "data received",
		unblock: func(tc *testConn, s *Stream) {
			tc.writeAckForAll()
		},
		success: true,
	}, {
		name: "stop sending received",
		unblock: func(tc *testConn, s *Stream) {
			tc.writeFrames(packetType1RTT, debugFrameStopSending{
				id: s.id,
			})
		},
	}, {
		name: "stream reset",
		unblock: func(tc *testConn, s *Stream) {
			s.Reset(0)
			tc.wait() // wait for test conn to process the Reset
		},
	}} {
		t.Run(test.name, func(t *testing.T) {
			tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters)
			data := make([]byte, 100)
			s.Write(data)
			s.Flush()
			tc.wantFrame("conn sends data for the stream",
				packetType1RTT, debugFrameStream{
					id:   s.id,
					data: data,
				})
			if err := s.Close(); err != context.Canceled {
				t.Fatalf("s.Close() = %v, want context.Canceled (data not acked yet)", err)
			}
			tc.wantFrame("conn sends FIN for closed stream",
				packetType1RTT, debugFrameStream{
					id:   s.id,
					off:  int64(len(data)),
					fin:  true,
					data: []byte{},
				})
			closing := runAsync(tc, func(ctx context.Context) (struct{}, error) {
				s.SetWriteContext(ctx)
				return struct{}{}, s.Close()
			})
			if _, err := closing.result(); err != errNotDone {
				t.Fatalf("s.Close() = %v, want it to block waiting for acks", err)
			}
			test.unblock(tc, s)
			_, err := closing.result()
			switch {
			case err == errNotDone:
				t.Fatalf("s.Close() still blocking; want it to have returned")
			case err == nil && !test.success:
				t.Fatalf("s.Close() = nil, want error")
			case err != nil && test.success:
				t.Fatalf("s.Close() = %v, want nil (all data acked)", err)
			}
		})
	}
}

func TestStreamCloseWriteWhenBlockedByStreamFlowControl(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters,
		func(p *transportParameters) {
			//p.initialMaxData = 0
			p.initialMaxStreamDataUni = 0
		})
	tc.ignoreFrame(frameTypeStreamDataBlocked)
	if _, err := s.Write([]byte{0, 1}); err != nil {
		t.Fatalf("s.Write = %v", err)
	}
	s.CloseWrite()
	tc.wantIdle("stream write is blocked by flow control")

	tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
		id:  s.id,
		max: 1,
	})
	tc.wantFrame("send data up to flow control limit",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: []byte{0},
		})
	tc.wantIdle("stream write is again blocked by flow control")

	tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
		id:  s.id,
		max: 2,
	})
	tc.wantFrame("send remaining data and FIN",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  1,
			data: []byte{1},
			fin:  true,
		})
}

func TestStreamPeerResetsWithUnreadAndUnsentData(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndRemoteStream(t, serverSide, styp)
		data := []byte{0, 1, 2, 3, 4, 5, 6, 7}
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data,
		})
		got := make([]byte, 4)
		if n, err := s.Read(got); n != len(got) || err != nil {
			t.Fatalf("Read start of stream: got %v, %v; want %v, nil", n, err, len(got))
		}
		const sentCode = 42
		tc.writeFrames(packetType1RTT, debugFrameResetStream{
			id:        s.id,
			finalSize: 20,
			code:      sentCode,
		})
		wantErr := StreamErrorCode(sentCode)
		if _, err := io.ReadAll(s); !errors.Is(err, wantErr) {
			t.Fatalf("Read reset stream: ReadAll got error %v; want %v", err, wantErr)
		}
	})
}

func TestStreamPeerResetWakesBlockedRead(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndRemoteStream(t, serverSide, styp)
		reader := runAsync(tc, func(ctx context.Context) (int, error) {
			s.SetReadContext(ctx)
			got := make([]byte, 4)
			return s.Read(got)
		})
		const sentCode = 42
		tc.writeFrames(packetType1RTT, debugFrameResetStream{
			id:        s.id,
			finalSize: 20,
			code:      sentCode,
		})
		wantErr := StreamErrorCode(sentCode)
		if n, err := reader.result(); n != 0 || !errors.Is(err, wantErr) {
			t.Fatalf("Read reset stream: got %v, %v; want 0, %v", n, err, wantErr)
		}
	})
}

func TestStreamPeerResetFollowedByData(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndRemoteStream(t, serverSide, styp)
		tc.writeFrames(packetType1RTT, debugFrameResetStream{
			id:        s.id,
			finalSize: 4,
			code:      1,
		})
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			data: []byte{0, 1, 2, 3},
		})
		// Another reset with a different code, for good measure.
		tc.writeFrames(packetType1RTT, debugFrameResetStream{
			id:        s.id,
			finalSize: 4,
			code:      2,
		})
		wantErr := StreamErrorCode(1)
		if n, err := s.Read(make([]byte, 16)); n != 0 || !errors.Is(err, wantErr) {
			t.Fatalf("Read from reset stream: got %v, %v; want 0, %v", n, err, wantErr)
		}
	})
}

func TestStreamResetInvalidCode(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters)
	s.Reset(1 << 62)
	tc.wantFrame("reset with invalid code sends a RESET_STREAM anyway",
		packetType1RTT, debugFrameResetStream{
			id: s.id,
			// The code we send here isn't specified,
			// so this could really be any value.
			code: (1 << 62) - 1,
		})
}

func TestStreamResetReceiveOnly(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream)
	s.Reset(0)
	tc.wantIdle("resetting a receive-only stream has no effect")
}

func TestStreamPeerStopSendingForActiveStream(t *testing.T) {
	// "An endpoint that receives a STOP_SENDING frame MUST send a RESET_STREAM frame if
	// the stream is in the "Ready" or "Send" state."
	// https://www.rfc-editor.org/rfc/rfc9000#section-3.5-4
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndLocalStream(t, serverSide, styp, permissiveTransportParameters)
		for i := 0; i < 4; i++ {
			s.Write([]byte{byte(i)})
			s.Flush()
			tc.wantFrame("write sends a STREAM frame to peer",
				packetType1RTT, debugFrameStream{
					id:   s.id,
					off:  int64(i),
					data: []byte{byte(i)},
				})
		}
		tc.writeFrames(packetType1RTT, debugFrameStopSending{
			id:   s.id,
			code: 42,
		})
		tc.wantFrame("receiving STOP_SENDING causes stream reset",
			packetType1RTT, debugFrameResetStream{
				id:        s.id,
				code:      42,
				finalSize: 4,
			})
		if n, err := s.Write([]byte{0}); err == nil {
			t.Errorf("s.Write() after STOP_SENDING = %v, %v; want error", n, err)
		}
		// This ack will result in some of the previous frames being marked as lost.
		tc.writeAckForLatest()
		tc.wantIdle("lost STREAM frames for reset stream are not resent")
	})
}

func TestStreamReceiveDataBlocked(t *testing.T) {
	tc := newTestConn(t, serverSide, permissiveTransportParameters)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	// We don't do anything with these frames,
	// but should accept them if the peer sends one.
	tc.writeFrames(packetType1RTT, debugFrameStreamDataBlocked{
		id:  newStreamID(clientSide, bidiStream, 0),
		max: 100,
	})
	tc.writeFrames(packetType1RTT, debugFrameDataBlocked{
		max: 100,
	})
	tc.wantIdle("no response to STREAM_DATA_BLOCKED and DATA_BLOCKED")
}

func TestStreamFlushExplicit(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc, s := newTestConnAndLocalStream(t, clientSide, styp, permissiveTransportParameters)
		want := []byte{0, 1, 2, 3}
		n, err := s.Write(want)
		if n != len(want) || err != nil {
			t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(want))
		}
		tc.wantIdle("unflushed data is not sent")
		s.Flush()
		tc.wantFrame("data is sent after flush",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				data: want,
			})
	})
}

func TestStreamFlushImplicitExact(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		const writeBufferSize = 4
		tc, s := newTestConnAndLocalStream(t, clientSide, styp,
			permissiveTransportParameters,
			func(c *Config) {
				c.MaxStreamWriteBufferSize = writeBufferSize
			})
		want := []byte{0, 1, 2, 3, 4, 5, 6}

		// This write doesn't quite fill the output buffer.
		n, err := s.Write(want[:3])
		if n != 3 || err != nil {
			t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(want))
		}
		tc.wantIdle("unflushed data is not sent")

		// This write fills the output buffer exactly.
		n, err = s.Write(want[3:4])
		if n != 1 || err != nil {
			t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(want))
		}
		tc.wantFrame("data is sent after write buffer fills",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				data: want[0:4],
			})
	})
}

func TestStreamFlushImplicitLargerThanBuffer(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		const writeBufferSize = 4
		tc, s := newTestConnAndLocalStream(t, clientSide, styp,
			permissiveTransportParameters,
			func(c *Config) {
				c.MaxStreamWriteBufferSize = writeBufferSize
			})
		want := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}

		w := runAsync(tc, func(ctx context.Context) (int, error) {
			s.SetWriteContext(ctx)
			n, err := s.Write(want)
			return n, err
		})

		tc.wantFrame("data is sent after write buffer fills",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				data: want[0:4],
			})
		tc.writeAckForAll()
		tc.wantFrame("ack permits sending more data",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  4,
				data: want[4:8],
			})
		tc.writeAckForAll()

		tc.wantIdle("write buffer is not full")
		if n, err := w.result(); n != len(want) || err != nil {
			t.Fatalf("Write() = %v, %v; want %v, nil", n, err, len(want))
		}

		s.Flush()
		tc.wantFrame("flush sends last buffer of data",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  8,
				data: want[8:],
			})
	})
}

type streamSide string

const (
	localStream  = streamSide("local")
	remoteStream = streamSide("remote")
)

func newTestConnAndStream(t *testing.T, side connSide, sside streamSide, styp streamType, opts ...any) (*testConn, *Stream) {
	if sside == localStream {
		return newTestConnAndLocalStream(t, side, styp, opts...)
	} else {
		return newTestConnAndRemoteStream(t, side, styp, opts...)
	}
}

func newTestConnAndLocalStream(t *testing.T, side connSide, styp streamType, opts ...any) (*testConn, *Stream) {
	t.Helper()
	tc := newTestConn(t, side, opts...)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	s := newLocalStream(t, tc, styp)
	s.SetReadContext(canceledContext())
	s.SetWriteContext(canceledContext())
	return tc, s
}

func newLocalStream(t *testing.T, tc *testConn, styp streamType) *Stream {
	t.Helper()
	ctx := canceledContext()
	s, err := tc.conn.newLocalStream(ctx, styp)
	if err != nil {
		t.Fatalf("conn.newLocalStream(%v) = %v", styp, err)
	}
	s.SetReadContext(canceledContext())
	s.SetWriteContext(canceledContext())
	return s
}

func newTestConnAndRemoteStream(t *testing.T, side connSide, styp streamType, opts ...any) (*testConn, *Stream) {
	t.Helper()
	tc := newTestConn(t, side, opts...)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	s := newRemoteStream(t, tc, styp)
	s.SetReadContext(canceledContext())
	s.SetWriteContext(canceledContext())
	return tc, s
}

func newRemoteStream(t *testing.T, tc *testConn, styp streamType) *Stream {
	t.Helper()
	ctx := canceledContext()
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id: newStreamID(tc.conn.side.peer(), styp, 0),
	})
	s, err := tc.conn.AcceptStream(ctx)
	if err != nil {
		t.Fatalf("conn.AcceptStream() = %v", err)
	}
	s.SetReadContext(canceledContext())
	s.SetWriteContext(canceledContext())
	return s
}

// permissiveTransportParameters may be passed as an option to newTestConn.
func permissiveTransportParameters(p *transportParameters) {
	p.initialMaxStreamsBidi = maxStreamsLimit
	p.initialMaxStreamsUni = maxStreamsLimit
	p.initialMaxData = maxVarint
	p.initialMaxStreamDataBidiRemote = maxVarint
	p.initialMaxStreamDataBidiLocal = maxVarint
	p.initialMaxStreamDataUni = maxVarint
}

func makeTestData(n int) []byte {
	b := make([]byte, n)
	for i := 0; i < n; i++ {
		b[i] = byte(i)
	}
	return b
}
