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
	"errors"
	"fmt"
	"math/big"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers"
)

//go:generate go run github.com/fjl/gencodec -type flatCallAction -field-override flatCallActionMarshaling -out gen_flatcallaction_json.go
//go:generate go run github.com/fjl/gencodec -type flatCallResult -field-override flatCallResultMarshaling -out gen_flatcallresult_json.go

func init() {
	tracers.DefaultDirectory.Register("flatCallTracer", newFlatCallTracer, false)
}

var parityErrorMapping = map[string]string{
	"contract creation code storage out of gas": "Out of gas",
	"out of gas":                      "Out of gas",
	"gas uint64 overflow":             "Out of gas",
	"max code size exceeded":          "Out of gas",
	"invalid jump destination":        "Bad jump destination",
	"execution reverted":              "Reverted",
	"return data out of bounds":       "Out of bounds",
	"stack limit reached 1024 (1023)": "Out of stack",
	"precompiled failed":              "Built-in failed",
	"invalid input length":            "Built-in failed",
}

var parityErrorMappingStartingWith = map[string]string{
	"invalid opcode:": "Bad instruction",
	"stack underflow": "Stack underflow",
}

// flatCallFrame is a standalone callframe.
type flatCallFrame struct {
	Action              flatCallAction  `json:"action"`
	BlockHash           *common.Hash    `json:"blockHash"`
	BlockNumber         uint64          `json:"blockNumber"`
	Error               string          `json:"error,omitempty"`
	Result              *flatCallResult `json:"result,omitempty"`
	Subtraces           int             `json:"subtraces"`
	TraceAddress        []int           `json:"traceAddress"`
	TransactionHash     *common.Hash    `json:"transactionHash"`
	TransactionPosition uint64          `json:"transactionPosition"`
	Type                string          `json:"type"`
}

type flatCallAction struct {
	Author         *common.Address `json:"author,omitempty"`
	RewardType     string          `json:"rewardType,omitempty"`
	SelfDestructed *common.Address `json:"address,omitempty"`
	Balance        *big.Int        `json:"balance,omitempty"`
	CallType       string          `json:"callType,omitempty"`
	CreationMethod string          `json:"creationMethod,omitempty"`
	From           *common.Address `json:"from,omitempty"`
	Gas            *uint64         `json:"gas,omitempty"`
	Init           *[]byte         `json:"init,omitempty"`
	Input          *[]byte         `json:"input,omitempty"`
	RefundAddress  *common.Address `json:"refundAddress,omitempty"`
	To             *common.Address `json:"to,omitempty"`
	Value          *big.Int        `json:"value,omitempty"`
}

type flatCallActionMarshaling struct {
	Balance *hexutil.Big
	Gas     *hexutil.Uint64
	Init    *hexutil.Bytes
	Input   *hexutil.Bytes
	Value   *hexutil.Big
}

type flatCallResult struct {
	Address *common.Address `json:"address,omitempty"`
	Code    *[]byte         `json:"code,omitempty"`
	GasUsed *uint64         `json:"gasUsed,omitempty"`
	Output  *[]byte         `json:"output,omitempty"`
}

type flatCallResultMarshaling struct {
	Code    *hexutil.Bytes
	GasUsed *hexutil.Uint64
	Output  *hexutil.Bytes
}

// flatCallTracer reports call frame information of a tx in a flat format, i.e.
// as opposed to the nested format of `callTracer`.
type flatCallTracer struct {
	tracer            *callTracer
	config            flatCallTracerConfig
	ctx               *tracers.Context // Holds tracer context data
	reason            error            // Textual reason for the interruption
	activePrecompiles []common.Address // Updated on CaptureStart based on given rules
}

type flatCallTracerConfig struct {
	ConvertParityErrors bool `json:"convertParityErrors"` // If true, call tracer converts errors to parity format
	IncludePrecompiles  bool `json:"includePrecompiles"`  // If true, call tracer includes calls to precompiled contracts
}

// newFlatCallTracer returns a new flatCallTracer.
func newFlatCallTracer(ctx *tracers.Context, cfg json.RawMessage) (tracers.Tracer, error) {
	var config flatCallTracerConfig
	if cfg != nil {
		if err := json.Unmarshal(cfg, &config); err != nil {
			return nil, err
		}
	}

	// Create inner call tracer with default configuration, don't forward
	// the OnlyTopCall or WithLog to inner for now
	tracer, err := tracers.DefaultDirectory.New("callTracer", ctx, nil)
	if err != nil {
		return nil, err
	}
	t, ok := tracer.(*callTracer)
	if !ok {
		return nil, errors.New("internal error: embedded tracer has wrong type")
	}

	return &flatCallTracer{tracer: t, ctx: ctx, config: config}, nil
}

// CaptureStart implements the EVMLogger interface to initialize the tracing operation.
func (t *flatCallTracer) CaptureStart(env *vm.EVM, from common.Address, to common.Address, create bool, input []byte, gas uint64, value *big.Int) {
	t.tracer.CaptureStart(env, from, to, create, input, gas, value)
	// Update list of precompiles based on current block
	rules := env.ChainConfig().Rules(env.Context.BlockNumber, env.Context.Random != nil, env.Context.Time)
	t.activePrecompiles = vm.ActivePrecompiles(rules)
}

// CaptureEnd is called after the call finishes to finalize the tracing.
func (t *flatCallTracer) CaptureEnd(output []byte, gasUsed uint64, err error) {
	t.tracer.CaptureEnd(output, gasUsed, err)
}

// CaptureState implements the EVMLogger interface to trace a single step of VM execution.
func (t *flatCallTracer) CaptureState(pc uint64, op vm.OpCode, gas, cost uint64, scope *vm.ScopeContext, rData []byte, depth int, err error) {
	t.tracer.CaptureState(pc, op, gas, cost, scope, rData, depth, err)
}

// CaptureFault implements the EVMLogger interface to trace an execution fault.
func (t *flatCallTracer) CaptureFault(pc uint64, op vm.OpCode, gas, cost uint64, scope *vm.ScopeContext, depth int, err error) {
	t.tracer.CaptureFault(pc, op, gas, cost, scope, depth, err)
}

// CaptureEnter is called when EVM enters a new scope (via call, create or selfdestruct).
func (t *flatCallTracer) CaptureEnter(typ vm.OpCode, from common.Address, to common.Address, input []byte, gas uint64, value *big.Int) {
	t.tracer.CaptureEnter(typ, from, to, input, gas, value)

	// Child calls must have a value, even if it's zero.
	// Practically speaking, only STATICCALL has nil value. Set it to zero.
	if t.tracer.callstack[len(t.tracer.callstack)-1].Value == nil && value == nil {
		t.tracer.callstack[len(t.tracer.callstack)-1].Value = big.NewInt(0)
	}
}

// CaptureExit is called when EVM exits a scope, even if the scope didn't
// execute any code.
func (t *flatCallTracer) CaptureExit(output []byte, gasUsed uint64, err error) {
	t.tracer.CaptureExit(output, gasUsed, err)

	// Parity traces don't include CALL/STATICCALLs to precompiles.
	// By default we remove them from the callstack.
	if t.config.IncludePrecompiles {
		return
	}
	var (
		// call has been nested in parent
		parent = t.tracer.callstack[len(t.tracer.callstack)-1]
		call   = parent.Calls[len(parent.Calls)-1]
		typ    = call.Type
		to     = call.To
	)
	if typ == vm.CALL || typ == vm.STATICCALL {
		if t.isPrecompiled(*to) {
			t.tracer.callstack[len(t.tracer.callstack)-1].Calls = parent.Calls[:len(parent.Calls)-1]
		}
	}
}

func (t *flatCallTracer) CaptureTxStart(gasLimit uint64) {
	t.tracer.CaptureTxStart(gasLimit)
}

func (t *flatCallTracer) CaptureTxEnd(restGas uint64) {
	t.tracer.CaptureTxEnd(restGas)
}

// GetResult returns an empty json object.
func (t *flatCallTracer) GetResult() (json.RawMessage, error) {
	if len(t.tracer.callstack) < 1 {
		return nil, errors.New("invalid number of calls")
	}

	flat, err := flatFromNested(&t.tracer.callstack[0], []int{}, t.config.ConvertParityErrors, t.ctx)
	if err != nil {
		return nil, err
	}

	res, err := json.Marshal(flat)
	if err != nil {
		return nil, err
	}
	return res, t.reason
}

// Stop terminates execution of the tracer at the first opportune moment.
func (t *flatCallTracer) Stop(err error) {
	t.tracer.Stop(err)
}

// isPrecompiled returns whether the addr is a precompile.
func (t *flatCallTracer) isPrecompiled(addr common.Address) bool {
	for _, p := range t.activePrecompiles {
		if p == addr {
			return true
		}
	}
	return false
}

func flatFromNested(input *callFrame, traceAddress []int, convertErrs bool, ctx *tracers.Context) (output []flatCallFrame, err error) {
	var frame *flatCallFrame
	switch input.Type {
	case vm.CREATE, vm.CREATE2:
		frame = newFlatCreate(input)
	case vm.SELFDESTRUCT:
		frame = newFlatSelfdestruct(input)
	case vm.CALL, vm.STATICCALL, vm.CALLCODE, vm.DELEGATECALL:
		frame = newFlatCall(input)
	default:
		return nil, fmt.Errorf("unrecognized call frame type: %s", input.Type)
	}

	frame.TraceAddress = traceAddress
	frame.Error = input.Error
	frame.Subtraces = len(input.Calls)
	fillCallFrameFromContext(frame, ctx)
	if convertErrs {
		convertErrorToParity(frame)
	}

	// Revert output contains useful information (revert reason).
	// Otherwise discard result.
	if input.Error != "" && input.Error != vm.ErrExecutionReverted.Error() {
		frame.Result = nil
	}

	output = append(output, *frame)
	if len(input.Calls) > 0 {
		for i, childCall := range input.Calls {
			childAddr := childTraceAddress(traceAddress, i)
			childCallCopy := childCall
			flat, err := flatFromNested(&childCallCopy, childAddr, convertErrs, ctx)
			if err != nil {
				return nil, err
			}
			output = append(output, flat...)
		}
	}

	return output, nil
}

func newFlatCreate(input *callFrame) *flatCallFrame {
	var (
		actionInit = input.Input[:]
		resultCode = input.Output[:]
	)

	return &flatCallFrame{
		Type: strings.ToLower(vm.CREATE.String()),
		Action: flatCallAction{
			From:  &input.From,
			Gas:   &input.Gas,
			Value: input.Value,
			Init:  &actionInit,
		},
		Result: &flatCallResult{
			GasUsed: &input.GasUsed,
			Address: input.To,
			Code:    &resultCode,
		},
	}
}

func newFlatCall(input *callFrame) *flatCallFrame {
	var (
		actionInput  = input.Input[:]
		resultOutput = input.Output[:]
	)

	return &flatCallFrame{
		Type: strings.ToLower(vm.CALL.String()),
		Action: flatCallAction{
			From:     &input.From,
			To:       input.To,
			Gas:      &input.Gas,
			Value:    input.Value,
			CallType: strings.ToLower(input.Type.String()),
			Input:    &actionInput,
		},
		Result: &flatCallResult{
			GasUsed: &input.GasUsed,
			Output:  &resultOutput,
		},
	}
}

func newFlatSelfdestruct(input *callFrame) *flatCallFrame {
	return &flatCallFrame{
		Type: "suicide",
		Action: flatCallAction{
			SelfDestructed: &input.From,
			Balance:        input.Value,
			RefundAddress:  input.To,
		},
	}
}

func fillCallFrameFromContext(callFrame *flatCallFrame, ctx *tracers.Context) {
	if ctx == nil {
		return
	}
	if ctx.BlockHash != (common.Hash{}) {
		callFrame.BlockHash = &ctx.BlockHash
	}
	if ctx.BlockNumber != nil {
		callFrame.BlockNumber = ctx.BlockNumber.Uint64()
	}
	if ctx.TxHash != (common.Hash{}) {
		callFrame.TransactionHash = &ctx.TxHash
	}
	callFrame.TransactionPosition = uint64(ctx.TxIndex)
}

func convertErrorToParity(call *flatCallFrame) {
	if call.Error == "" {
		return
	}

	if parityError, ok := parityErrorMapping[call.Error]; ok {
		call.Error = parityError
	} else {
		for gethError, parityError := range parityErrorMappingStartingWith {
			if strings.HasPrefix(call.Error, gethError) {
				call.Error = parityError
			}
		}
	}
}

func childTraceAddress(a []int, i int) []int {
	child := make([]int, 0, len(a)+1)
	child = append(child, a...)
	child = append(child, i)
	return child
}
