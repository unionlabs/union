/*
 *
 * Copyright 2024 gRPC authors.
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

package grpchttp2

import (
	"io"

	"golang.org/x/net/http2"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc/mem"
)

// FramerBridge adapts the net/x/http2 Framer to satisfy the grpchttp2.Framer
// interface.
//
// Note: This allows temporary use of the older framer and will be removed in
// a future release after the new framer stabilizes.
type FramerBridge struct {
	framer *http2.Framer  // the underlying http2.Framer implementation to perform reads and writes.
	pool   mem.BufferPool // a pool to reuse buffers when reading.
}

// NewFramerBridge creates an adaptor that wraps a http2.Framer in a
// grpchttp2.Framer.
//
// Internally, it creates a http2.Framer that uses the provided io.Reader and
// io.Writer, and is configured with a maximum header list size of
// maxHeaderListSize.
//
// Frames returned by a call to the underlying http2.Framer's ReadFrame() method
// need to be consumed before the next call to it. To overcome this restriction,
// the data in a Frame returned by the http2.Framer's ReadFrame is copied into a
// buffer from the given pool. If no pool is provided, a default pool provided
// by the mem package is used.
func NewFramerBridge(w io.Writer, r io.Reader, maxHeaderListSize uint32, pool mem.BufferPool) *FramerBridge {
	fr := http2.NewFramer(w, r)
	fr.SetReuseFrames()
	fr.MaxHeaderListSize = maxHeaderListSize
	fr.ReadMetaHeaders = hpack.NewDecoder(initHeaderTableSize, nil)

	if pool == nil {
		pool = mem.DefaultBufferPool()
	}

	return &FramerBridge{
		framer: fr,
		pool:   pool,
	}
}

// ReadFrame reads a frame from the underlying http2.Framer and returns a
// Frame defined in the grpchttp2 package. This operation copies the data to a
// buffer from the pool, making it safe to use even after another call to
// ReadFrame.
func (fr *FramerBridge) ReadFrame() (Frame, error) {
	f, err := fr.framer.ReadFrame()
	if err != nil {
		return nil, err
	}

	h := f.Header()
	hdr := &FrameHeader{
		Size:     h.Length,
		Type:     FrameType(h.Type),
		Flags:    Flag(h.Flags),
		StreamID: h.StreamID,
	}

	switch f := f.(type) {
	case *http2.DataFrame:
		buf := fr.pool.Get(int(hdr.Size))
		copy(*buf, f.Data())
		return &DataFrame{
			hdr:  hdr,
			Data: *buf,
			free: func() { fr.pool.Put(buf) },
		}, nil
	case *http2.RSTStreamFrame:
		return &RSTStreamFrame{
			hdr:  hdr,
			Code: ErrCode(f.ErrCode),
		}, nil
	case *http2.SettingsFrame:
		buf := make([]Setting, 0, f.NumSettings())
		f.ForeachSetting(func(s http2.Setting) error {
			buf = append(buf, Setting{
				ID:    SettingID(s.ID),
				Value: s.Val,
			})
			return nil
		})
		return &SettingsFrame{
			hdr:      hdr,
			Settings: buf,
		}, nil
	case *http2.PingFrame:
		buf := fr.pool.Get(int(hdr.Size))
		copy(*buf, f.Data[:])
		return &PingFrame{
			hdr:  hdr,
			Data: *buf,
			free: func() { fr.pool.Put(buf) },
		}, nil
	case *http2.GoAwayFrame:
		// Size of the frame minus the code and lastStreamID
		buf := fr.pool.Get(int(hdr.Size) - 8)
		copy(*buf, f.DebugData())
		return &GoAwayFrame{
			hdr:          hdr,
			LastStreamID: f.LastStreamID,
			Code:         ErrCode(f.ErrCode),
			DebugData:    *buf,
			free:         func() { fr.pool.Put(buf) },
		}, nil
	case *http2.WindowUpdateFrame:
		return &WindowUpdateFrame{
			hdr: hdr,
			Inc: f.Increment,
		}, nil
	case *http2.MetaHeadersFrame:
		return &MetaHeadersFrame{
			hdr:    hdr,
			Fields: f.Fields,
		}, nil
	default:
		buf := fr.pool.Get(int(hdr.Size))
		uf := f.(*http2.UnknownFrame)
		copy(*buf, uf.Payload())
		return &UnknownFrame{
			hdr:     hdr,
			Payload: *buf,
			free:    func() { fr.pool.Put(buf) },
		}, nil
	}
}

// WriteData writes a DATA Frame into the underlying writer.
func (fr *FramerBridge) WriteData(streamID uint32, endStream bool, data ...[]byte) error {
	if len(data) == 1 {
		return fr.framer.WriteData(streamID, endStream, data[0])
	}

	tl := 0
	for _, s := range data {
		tl += len(s)
	}

	buf := fr.pool.Get(tl)
	*buf = (*buf)[:0]
	defer fr.pool.Put(buf)
	for _, s := range data {
		*buf = append(*buf, s...)
	}

	return fr.framer.WriteData(streamID, endStream, *buf)
}

// WriteHeaders writes a Headers Frame into the underlying writer.
func (fr *FramerBridge) WriteHeaders(streamID uint32, endStream, endHeaders bool, headerBlock []byte) error {
	return fr.framer.WriteHeaders(http2.HeadersFrameParam{
		StreamID:      streamID,
		EndStream:     endStream,
		EndHeaders:    endHeaders,
		BlockFragment: headerBlock,
	})
}

// WriteRSTStream writes a RSTStream Frame into the underlying writer.
func (fr *FramerBridge) WriteRSTStream(streamID uint32, code ErrCode) error {
	return fr.framer.WriteRSTStream(streamID, http2.ErrCode(code))
}

// WriteSettings writes a Settings Frame into the underlying writer.
func (fr *FramerBridge) WriteSettings(settings ...Setting) error {
	ss := make([]http2.Setting, 0, len(settings))
	for _, s := range settings {
		ss = append(ss, http2.Setting{
			ID:  http2.SettingID(s.ID),
			Val: s.Value,
		})
	}

	return fr.framer.WriteSettings(ss...)
}

// WriteSettingsAck writes a Settings Frame with the Ack flag set.
func (fr *FramerBridge) WriteSettingsAck() error {
	return fr.framer.WriteSettingsAck()
}

// WritePing writes a Ping frame to the underlying writer.
func (fr *FramerBridge) WritePing(ack bool, data [8]byte) error {
	return fr.framer.WritePing(ack, data)
}

// WriteGoAway writes a GoAway Frame to the unerlying writer.
func (fr *FramerBridge) WriteGoAway(maxStreamID uint32, code ErrCode, debugData []byte) error {
	return fr.framer.WriteGoAway(maxStreamID, http2.ErrCode(code), debugData)
}

// WriteWindowUpdate writes a WindowUpdate Frame into the underlying writer.
func (fr *FramerBridge) WriteWindowUpdate(streamID, inc uint32) error {
	return fr.framer.WriteWindowUpdate(streamID, inc)
}

// WriteContinuation writes a Continuation Frame into the underlying writer.
func (fr *FramerBridge) WriteContinuation(streamID uint32, endHeaders bool, headerBlock []byte) error {
	return fr.framer.WriteContinuation(streamID, endHeaders, headerBlock)
}
