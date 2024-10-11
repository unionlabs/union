// Copyright 2022 The go-ethereum Authors
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

package engine

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/rpc"
)

// EngineAPIError is a standardized error message between consensus and execution
// clients, also containing any custom error message Geth might include.
type EngineAPIError struct {
	code int
	msg  string
	err  error
}

func (e *EngineAPIError) ErrorCode() int { return e.code }
func (e *EngineAPIError) Error() string  { return e.msg }
func (e *EngineAPIError) ErrorData() interface{} {
	if e.err == nil {
		return nil
	}
	return struct {
		Error string `json:"err"`
	}{e.err.Error()}
}

// With returns a copy of the error with a new embedded custom data field.
func (e *EngineAPIError) With(err error) *EngineAPIError {
	return &EngineAPIError{
		code: e.code,
		msg:  e.msg,
		err:  err,
	}
}

var (
	_ rpc.Error     = new(EngineAPIError)
	_ rpc.DataError = new(EngineAPIError)
)

var (
	// VALID is returned by the engine API in the following calls:
	//   - newPayloadV1:       if the payload was already known or was just validated and executed
	//   - forkchoiceUpdateV1: if the chain accepted the reorg (might ignore if it's stale)
	VALID = "VALID"

	// INVALID is returned by the engine API in the following calls:
	//   - newPayloadV1:       if the payload failed to execute on top of the local chain
	//   - forkchoiceUpdateV1: if the new head is unknown, pre-merge, or reorg to it fails
	INVALID = "INVALID"

	// SYNCING is returned by the engine API in the following calls:
	//   - newPayloadV1:       if the payload was accepted on top of an active sync
	//   - forkchoiceUpdateV1: if the new head was seen before, but not part of the chain
	SYNCING = "SYNCING"

	// ACCEPTED is returned by the engine API in the following calls:
	//   - newPayloadV1: if the payload was accepted, but not processed (side chain)
	ACCEPTED = "ACCEPTED"

	GenericServerError       = &EngineAPIError{code: -32000, msg: "Server error"}
	UnknownPayload           = &EngineAPIError{code: -38001, msg: "Unknown payload"}
	InvalidForkChoiceState   = &EngineAPIError{code: -38002, msg: "Invalid forkchoice state"}
	InvalidPayloadAttributes = &EngineAPIError{code: -38003, msg: "Invalid payload attributes"}
	TooLargeRequest          = &EngineAPIError{code: -38004, msg: "Too large request"}
	InvalidParams            = &EngineAPIError{code: -32602, msg: "Invalid parameters"}
	UnsupportedFork          = &EngineAPIError{code: -38005, msg: "Unsupported fork"}

	STATUS_INVALID         = ForkChoiceResponse{PayloadStatus: PayloadStatusV1{Status: INVALID}, PayloadID: nil}
	STATUS_SYNCING         = ForkChoiceResponse{PayloadStatus: PayloadStatusV1{Status: SYNCING}, PayloadID: nil}
	INVALID_TERMINAL_BLOCK = PayloadStatusV1{Status: INVALID, LatestValidHash: &common.Hash{}}
)
