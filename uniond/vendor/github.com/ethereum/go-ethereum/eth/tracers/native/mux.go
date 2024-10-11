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

package native

import (
	"encoding/json"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers"
)

func init() {
	tracers.DefaultDirectory.Register("muxTracer", newMuxTracer, false)
}

// muxTracer is a go implementation of the Tracer interface which
// runs multiple tracers in one go.
type muxTracer struct {
	names   []string
	tracers []tracers.Tracer
}

// newMuxTracer returns a new mux tracer.
func newMuxTracer(ctx *tracers.Context, cfg json.RawMessage) (tracers.Tracer, error) {
	var config map[string]json.RawMessage
	if cfg != nil {
		if err := json.Unmarshal(cfg, &config); err != nil {
			return nil, err
		}
	}
	objects := make([]tracers.Tracer, 0, len(config))
	names := make([]string, 0, len(config))
	for k, v := range config {
		t, err := tracers.DefaultDirectory.New(k, ctx, v)
		if err != nil {
			return nil, err
		}
		objects = append(objects, t)
		names = append(names, k)
	}

	return &muxTracer{names: names, tracers: objects}, nil
}

// CaptureStart implements the EVMLogger interface to initialize the tracing operation.
func (t *muxTracer) CaptureStart(env *vm.EVM, from common.Address, to common.Address, create bool, input []byte, gas uint64, value *big.Int) {
	for _, t := range t.tracers {
		t.CaptureStart(env, from, to, create, input, gas, value)
	}
}

// CaptureEnd is called after the call finishes to finalize the tracing.
func (t *muxTracer) CaptureEnd(output []byte, gasUsed uint64, err error) {
	for _, t := range t.tracers {
		t.CaptureEnd(output, gasUsed, err)
	}
}

// CaptureState implements the EVMLogger interface to trace a single step of VM execution.
func (t *muxTracer) CaptureState(pc uint64, op vm.OpCode, gas, cost uint64, scope *vm.ScopeContext, rData []byte, depth int, err error) {
	for _, t := range t.tracers {
		t.CaptureState(pc, op, gas, cost, scope, rData, depth, err)
	}
}

// CaptureFault implements the EVMLogger interface to trace an execution fault.
func (t *muxTracer) CaptureFault(pc uint64, op vm.OpCode, gas, cost uint64, scope *vm.ScopeContext, depth int, err error) {
	for _, t := range t.tracers {
		t.CaptureFault(pc, op, gas, cost, scope, depth, err)
	}
}

// CaptureEnter is called when EVM enters a new scope (via call, create or selfdestruct).
func (t *muxTracer) CaptureEnter(typ vm.OpCode, from common.Address, to common.Address, input []byte, gas uint64, value *big.Int) {
	for _, t := range t.tracers {
		t.CaptureEnter(typ, from, to, input, gas, value)
	}
}

// CaptureExit is called when EVM exits a scope, even if the scope didn't
// execute any code.
func (t *muxTracer) CaptureExit(output []byte, gasUsed uint64, err error) {
	for _, t := range t.tracers {
		t.CaptureExit(output, gasUsed, err)
	}
}

func (t *muxTracer) CaptureTxStart(gasLimit uint64) {
	for _, t := range t.tracers {
		t.CaptureTxStart(gasLimit)
	}
}

func (t *muxTracer) CaptureTxEnd(restGas uint64) {
	for _, t := range t.tracers {
		t.CaptureTxEnd(restGas)
	}
}

// GetResult returns an empty json object.
func (t *muxTracer) GetResult() (json.RawMessage, error) {
	resObject := make(map[string]json.RawMessage)
	for i, tt := range t.tracers {
		r, err := tt.GetResult()
		if err != nil {
			return nil, err
		}
		resObject[t.names[i]] = r
	}
	res, err := json.Marshal(resObject)
	if err != nil {
		return nil, err
	}
	return res, nil
}

// Stop terminates execution of the tracer at the first opportune moment.
func (t *muxTracer) Stop(err error) {
	for _, t := range t.tracers {
		t.Stop(err)
	}
}
