// Copyright 2021 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package vflux

import (
	"errors"
	"math"
	"math/big"

	"github.com/ethereum/go-ethereum/rlp"
)

var ErrNoReply = errors.New("no reply for given request")

const (
	MaxRequestLength    = 16 // max number of individual requests in a batch
	CapacityQueryName   = "cq"
	CapacityQueryMaxLen = 16
)

type (
	// Request describes a single vflux request inside a batch. Service and request
	// type are identified by strings, parameters are RLP encoded.
	Request struct {
		Service, Name string
		Params        []byte
	}
	// Requests are a batch of vflux requests
	Requests []Request

	// Replies are the replies to a batch of requests
	Replies [][]byte

	// CapacityQueryReq is the encoding format of the capacity query
	CapacityQueryReq struct {
		Bias      uint64 // seconds
		AddTokens []IntOrInf
	}
	// CapacityQueryReply is the encoding format of the response to the capacity query
	CapacityQueryReply []uint64
)

// Add encodes and adds a new request to the batch
func (r *Requests) Add(service, name string, val interface{}) (int, error) {
	enc, err := rlp.EncodeToBytes(val)
	if err != nil {
		return -1, err
	}
	*r = append(*r, Request{
		Service: service,
		Name:    name,
		Params:  enc,
	})
	return len(*r) - 1, nil
}

// Get decodes the reply to the i-th request in the batch
func (r Replies) Get(i int, val interface{}) error {
	if i < 0 || i >= len(r) {
		return ErrNoReply
	}
	return rlp.DecodeBytes(r[i], val)
}

const (
	IntNonNegative = iota
	IntNegative
	IntPlusInf
	IntMinusInf
)

// IntOrInf is the encoding format for arbitrary length signed integers that can also
// hold the values of +Inf or -Inf
type IntOrInf struct {
	Type  uint8
	Value big.Int
}

// BigInt returns the value as a big.Int or panics if the value is infinity
func (i *IntOrInf) BigInt() *big.Int {
	switch i.Type {
	case IntNonNegative:
		return new(big.Int).Set(&i.Value)
	case IntNegative:
		return new(big.Int).Neg(&i.Value)
	case IntPlusInf:
		panic(nil) // caller should check Inf() before trying to convert to big.Int
	case IntMinusInf:
		panic(nil)
	}
	return &big.Int{} // invalid type decodes to 0 value
}

// Inf returns 1 if the value is +Inf, -1 if it is -Inf, 0 otherwise
func (i *IntOrInf) Inf() int {
	switch i.Type {
	case IntPlusInf:
		return 1
	case IntMinusInf:
		return -1
	}
	return 0 // invalid type decodes to 0 value
}

// Int64 limits the value between MinInt64 and MaxInt64 (even if it is +-Inf) and returns an int64 type
func (i *IntOrInf) Int64() int64 {
	switch i.Type {
	case IntNonNegative:
		if i.Value.IsInt64() {
			return i.Value.Int64()
		} else {
			return math.MaxInt64
		}
	case IntNegative:
		if i.Value.IsInt64() {
			return -i.Value.Int64()
		} else {
			return math.MinInt64
		}
	case IntPlusInf:
		return math.MaxInt64
	case IntMinusInf:
		return math.MinInt64
	}
	return 0 // invalid type decodes to 0 value
}

// SetBigInt sets the value to the given big.Int
func (i *IntOrInf) SetBigInt(v *big.Int) {
	if v.Sign() >= 0 {
		i.Type = IntNonNegative
		i.Value.Set(v)
	} else {
		i.Type = IntNegative
		i.Value.Neg(v)
	}
}

// SetInt64 sets the value to the given int64. Note that MaxInt64 translates to +Inf
// while MinInt64 translates to -Inf.
func (i *IntOrInf) SetInt64(v int64) {
	if v >= 0 {
		if v == math.MaxInt64 {
			i.Type = IntPlusInf
		} else {
			i.Type = IntNonNegative
			i.Value.SetInt64(v)
		}
	} else {
		if v == math.MinInt64 {
			i.Type = IntMinusInf
		} else {
			i.Type = IntNegative
			i.Value.SetInt64(-v)
		}
	}
}

// SetInf sets the value to +Inf or -Inf
func (i *IntOrInf) SetInf(sign int) {
	if sign == 1 {
		i.Type = IntPlusInf
	} else {
		i.Type = IntMinusInf
	}
}
