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

package native

import (
	"encoding/json"
	"math/big"
	"strconv"
	"sync/atomic"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers"
)

func init() {
	tracers.DefaultDirectory.Register("4byteTracer", newFourByteTracer, false)
}

// fourByteTracer searches for 4byte-identifiers, and collects them for post-processing.
// It collects the methods identifiers along with the size of the supplied data, so
// a reversed signature can be matched against the size of the data.
//
// Example:
//
//	> debug.traceTransaction( "0x214e597e35da083692f5386141e69f47e973b2c56e7a8073b1ea08fd7571e9de", {tracer: "4byteTracer"})
//	{
//	  0x27dc297e-128: 1,
//	  0x38cc4831-0: 2,
//	  0x524f3889-96: 1,
//	  0xadf59f99-288: 1,
//	  0xc281d19e-0: 1
//	}
type fourByteTracer struct {
	noopTracer
	ids               map[string]int   // ids aggregates the 4byte ids found
	interrupt         atomic.Bool      // Atomic flag to signal execution interruption
	reason            error            // Textual reason for the interruption
	activePrecompiles []common.Address // Updated on CaptureStart based on given rules
}

// newFourByteTracer returns a native go tracer which collects
// 4 byte-identifiers of a tx, and implements vm.EVMLogger.
func newFourByteTracer(ctx *tracers.Context, _ json.RawMessage) (tracers.Tracer, error) {
	t := &fourByteTracer{
		ids: make(map[string]int),
	}
	return t, nil
}

// isPrecompiled returns whether the addr is a precompile. Logic borrowed from newJsTracer in eth/tracers/js/tracer.go
func (t *fourByteTracer) isPrecompiled(addr common.Address) bool {
	for _, p := range t.activePrecompiles {
		if p == addr {
			return true
		}
	}
	return false
}

// store saves the given identifier and datasize.
func (t *fourByteTracer) store(id []byte, size int) {
	key := bytesToHex(id) + "-" + strconv.Itoa(size)
	t.ids[key] += 1
}

// CaptureStart implements the EVMLogger interface to initialize the tracing operation.
func (t *fourByteTracer) CaptureStart(env *vm.EVM, from common.Address, to common.Address, create bool, input []byte, gas uint64, value *big.Int) {
	// Update list of precompiles based on current block
	rules := env.ChainConfig().Rules(env.Context.BlockNumber, env.Context.Random != nil, env.Context.Time)
	t.activePrecompiles = vm.ActivePrecompiles(rules)

	// Save the outer calldata also
	if len(input) >= 4 {
		t.store(input[0:4], len(input)-4)
	}
}

// CaptureEnter is called when EVM enters a new scope (via call, create or selfdestruct).
func (t *fourByteTracer) CaptureEnter(op vm.OpCode, from common.Address, to common.Address, input []byte, gas uint64, value *big.Int) {
	// Skip if tracing was interrupted
	if t.interrupt.Load() {
		return
	}
	if len(input) < 4 {
		return
	}
	// primarily we want to avoid CREATE/CREATE2/SELFDESTRUCT
	if op != vm.DELEGATECALL && op != vm.STATICCALL &&
		op != vm.CALL && op != vm.CALLCODE {
		return
	}
	// Skip any pre-compile invocations, those are just fancy opcodes
	if t.isPrecompiled(to) {
		return
	}
	t.store(input[0:4], len(input)-4)
}

// GetResult returns the json-encoded nested list of call traces, and any
// error arising from the encoding or forceful termination (via `Stop`).
func (t *fourByteTracer) GetResult() (json.RawMessage, error) {
	res, err := json.Marshal(t.ids)
	if err != nil {
		return nil, err
	}
	return res, t.reason
}

// Stop terminates execution of the tracer at the first opportune moment.
func (t *fourByteTracer) Stop(err error) {
	t.reason = err
	t.interrupt.Store(true)
}

func bytesToHex(s []byte) string {
	return "0x" + common.Bytes2Hex(s)
}
