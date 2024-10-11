// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"encoding/binary"
	"net/netip"
	"time"
)

// transportParameters transferred in the quic_transport_parameters TLS extension.
// https://www.rfc-editor.org/rfc/rfc9000.html#section-18.2
type transportParameters struct {
	originalDstConnID              []byte
	maxIdleTimeout                 time.Duration
	statelessResetToken            []byte
	maxUDPPayloadSize              int64
	initialMaxData                 int64
	initialMaxStreamDataBidiLocal  int64
	initialMaxStreamDataBidiRemote int64
	initialMaxStreamDataUni        int64
	initialMaxStreamsBidi          int64
	initialMaxStreamsUni           int64
	ackDelayExponent               int8
	maxAckDelay                    time.Duration
	disableActiveMigration         bool
	preferredAddrV4                netip.AddrPort
	preferredAddrV6                netip.AddrPort
	preferredAddrConnID            []byte
	preferredAddrResetToken        []byte
	activeConnIDLimit              int64
	initialSrcConnID               []byte
	retrySrcConnID                 []byte
}

const (
	defaultParamMaxUDPPayloadSize       = 65527
	defaultParamAckDelayExponent        = 3
	defaultParamMaxAckDelayMilliseconds = 25
	defaultParamActiveConnIDLimit       = 2
)

// defaultTransportParameters is initialized to the RFC 9000 default values.
func defaultTransportParameters() transportParameters {
	return transportParameters{
		maxUDPPayloadSize: defaultParamMaxUDPPayloadSize,
		ackDelayExponent:  defaultParamAckDelayExponent,
		maxAckDelay:       defaultParamMaxAckDelayMilliseconds * time.Millisecond,
		activeConnIDLimit: defaultParamActiveConnIDLimit,
	}
}

const (
	paramOriginalDestinationConnectionID = 0x00
	paramMaxIdleTimeout                  = 0x01
	paramStatelessResetToken             = 0x02
	paramMaxUDPPayloadSize               = 0x03
	paramInitialMaxData                  = 0x04
	paramInitialMaxStreamDataBidiLocal   = 0x05
	paramInitialMaxStreamDataBidiRemote  = 0x06
	paramInitialMaxStreamDataUni         = 0x07
	paramInitialMaxStreamsBidi           = 0x08
	paramInitialMaxStreamsUni            = 0x09
	paramAckDelayExponent                = 0x0a
	paramMaxAckDelay                     = 0x0b
	paramDisableActiveMigration          = 0x0c
	paramPreferredAddress                = 0x0d
	paramActiveConnectionIDLimit         = 0x0e
	paramInitialSourceConnectionID       = 0x0f
	paramRetrySourceConnectionID         = 0x10
)

func marshalTransportParameters(p transportParameters) []byte {
	var b []byte
	if v := p.originalDstConnID; v != nil {
		b = appendVarint(b, paramOriginalDestinationConnectionID)
		b = appendVarintBytes(b, v)
	}
	if v := uint64(p.maxIdleTimeout / time.Millisecond); v != 0 {
		b = appendVarint(b, paramMaxIdleTimeout)
		b = appendVarint(b, uint64(sizeVarint(v)))
		b = appendVarint(b, uint64(v))
	}
	if v := p.statelessResetToken; v != nil {
		b = appendVarint(b, paramStatelessResetToken)
		b = appendVarintBytes(b, v)
	}
	if v := p.maxUDPPayloadSize; v != defaultParamMaxUDPPayloadSize {
		b = appendVarint(b, paramMaxUDPPayloadSize)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxData; v != 0 {
		b = appendVarint(b, paramInitialMaxData)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxStreamDataBidiLocal; v != 0 {
		b = appendVarint(b, paramInitialMaxStreamDataBidiLocal)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxStreamDataBidiRemote; v != 0 {
		b = appendVarint(b, paramInitialMaxStreamDataBidiRemote)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxStreamDataUni; v != 0 {
		b = appendVarint(b, paramInitialMaxStreamDataUni)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxStreamsBidi; v != 0 {
		b = appendVarint(b, paramInitialMaxStreamsBidi)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialMaxStreamsUni; v != 0 {
		b = appendVarint(b, paramInitialMaxStreamsUni)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.ackDelayExponent; v != defaultParamAckDelayExponent {
		b = appendVarint(b, paramAckDelayExponent)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := uint64(p.maxAckDelay / time.Millisecond); v != defaultParamMaxAckDelayMilliseconds {
		b = appendVarint(b, paramMaxAckDelay)
		b = appendVarint(b, uint64(sizeVarint(v)))
		b = appendVarint(b, v)
	}
	if p.disableActiveMigration {
		b = appendVarint(b, paramDisableActiveMigration)
		b = append(b, 0) // 0-length value
	}
	if p.preferredAddrConnID != nil {
		b = append(b, paramPreferredAddress)
		b = appendVarint(b, uint64(4+2+16+2+1+len(p.preferredAddrConnID)+16))
		b = append(b, p.preferredAddrV4.Addr().AsSlice()...)           // 4 bytes
		b = binary.BigEndian.AppendUint16(b, p.preferredAddrV4.Port()) // 2 bytes
		b = append(b, p.preferredAddrV6.Addr().AsSlice()...)           // 16 bytes
		b = binary.BigEndian.AppendUint16(b, p.preferredAddrV6.Port()) // 2 bytes
		b = appendUint8Bytes(b, p.preferredAddrConnID)                 // 1 byte + len(conn_id)
		b = append(b, p.preferredAddrResetToken...)                    // 16 bytes
	}
	if v := p.activeConnIDLimit; v != defaultParamActiveConnIDLimit {
		b = appendVarint(b, paramActiveConnectionIDLimit)
		b = appendVarint(b, uint64(sizeVarint(uint64(v))))
		b = appendVarint(b, uint64(v))
	}
	if v := p.initialSrcConnID; v != nil {
		b = appendVarint(b, paramInitialSourceConnectionID)
		b = appendVarintBytes(b, v)
	}
	if v := p.retrySrcConnID; v != nil {
		b = appendVarint(b, paramRetrySourceConnectionID)
		b = appendVarintBytes(b, v)
	}
	return b
}

func unmarshalTransportParams(params []byte) (transportParameters, error) {
	p := defaultTransportParameters()
	for len(params) > 0 {
		id, n := consumeVarint(params)
		if n < 0 {
			return p, localTransportError{code: errTransportParameter}
		}
		params = params[n:]
		val, n := consumeVarintBytes(params)
		if n < 0 {
			return p, localTransportError{code: errTransportParameter}
		}
		params = params[n:]
		n = 0
		switch id {
		case paramOriginalDestinationConnectionID:
			p.originalDstConnID = val
			n = len(val)
		case paramMaxIdleTimeout:
			var v uint64
			v, n = consumeVarint(val)
			// If this is unreasonably large, consider it as no timeout to avoid
			// time.Duration overflows.
			if v > 1<<32 {
				v = 0
			}
			p.maxIdleTimeout = time.Duration(v) * time.Millisecond
		case paramStatelessResetToken:
			if len(val) != 16 {
				return p, localTransportError{code: errTransportParameter}
			}
			p.statelessResetToken = val
			n = 16
		case paramMaxUDPPayloadSize:
			p.maxUDPPayloadSize, n = consumeVarintInt64(val)
			if p.maxUDPPayloadSize < 1200 {
				return p, localTransportError{code: errTransportParameter}
			}
		case paramInitialMaxData:
			p.initialMaxData, n = consumeVarintInt64(val)
		case paramInitialMaxStreamDataBidiLocal:
			p.initialMaxStreamDataBidiLocal, n = consumeVarintInt64(val)
		case paramInitialMaxStreamDataBidiRemote:
			p.initialMaxStreamDataBidiRemote, n = consumeVarintInt64(val)
		case paramInitialMaxStreamDataUni:
			p.initialMaxStreamDataUni, n = consumeVarintInt64(val)
		case paramInitialMaxStreamsBidi:
			p.initialMaxStreamsBidi, n = consumeVarintInt64(val)
			if p.initialMaxStreamsBidi > maxStreamsLimit {
				return p, localTransportError{code: errTransportParameter}
			}
		case paramInitialMaxStreamsUni:
			p.initialMaxStreamsUni, n = consumeVarintInt64(val)
			if p.initialMaxStreamsUni > maxStreamsLimit {
				return p, localTransportError{code: errTransportParameter}
			}
		case paramAckDelayExponent:
			var v uint64
			v, n = consumeVarint(val)
			if v > 20 {
				return p, localTransportError{code: errTransportParameter}
			}
			p.ackDelayExponent = int8(v)
		case paramMaxAckDelay:
			var v uint64
			v, n = consumeVarint(val)
			if v >= 1<<14 {
				return p, localTransportError{code: errTransportParameter}
			}
			p.maxAckDelay = time.Duration(v) * time.Millisecond
		case paramDisableActiveMigration:
			p.disableActiveMigration = true
		case paramPreferredAddress:
			if len(val) < 4+2+16+2+1 {
				return p, localTransportError{code: errTransportParameter}
			}
			p.preferredAddrV4 = netip.AddrPortFrom(
				netip.AddrFrom4(*(*[4]byte)(val[:4])),
				binary.BigEndian.Uint16(val[4:][:2]),
			)
			val = val[4+2:]
			p.preferredAddrV6 = netip.AddrPortFrom(
				netip.AddrFrom16(*(*[16]byte)(val[:16])),
				binary.BigEndian.Uint16(val[16:][:2]),
			)
			val = val[16+2:]
			var nn int
			p.preferredAddrConnID, nn = consumeUint8Bytes(val)
			if nn < 0 {
				return p, localTransportError{code: errTransportParameter}
			}
			val = val[nn:]
			if len(val) != 16 {
				return p, localTransportError{code: errTransportParameter}
			}
			p.preferredAddrResetToken = val
			val = nil
		case paramActiveConnectionIDLimit:
			p.activeConnIDLimit, n = consumeVarintInt64(val)
			if p.activeConnIDLimit < 2 {
				return p, localTransportError{code: errTransportParameter}
			}
		case paramInitialSourceConnectionID:
			p.initialSrcConnID = val
			n = len(val)
		case paramRetrySourceConnectionID:
			p.retrySrcConnID = val
			n = len(val)
		default:
			n = len(val)
		}
		if n != len(val) {
			return p, localTransportError{code: errTransportParameter}
		}
	}
	return p, nil
}
