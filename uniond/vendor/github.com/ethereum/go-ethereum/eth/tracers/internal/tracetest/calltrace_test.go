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

package tracetest

import (
	"encoding/json"
	"math/big"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/tests"
)

type callContext struct {
	Number     math.HexOrDecimal64   `json:"number"`
	Difficulty *math.HexOrDecimal256 `json:"difficulty"`
	Time       math.HexOrDecimal64   `json:"timestamp"`
	GasLimit   math.HexOrDecimal64   `json:"gasLimit"`
	Miner      common.Address        `json:"miner"`
}

// callLog is the result of LOG opCode
type callLog struct {
	Address common.Address `json:"address"`
	Topics  []common.Hash  `json:"topics"`
	Data    hexutil.Bytes  `json:"data"`
}

// callTrace is the result of a callTracer run.
type callTrace struct {
	From         common.Address  `json:"from"`
	Gas          *hexutil.Uint64 `json:"gas"`
	GasUsed      *hexutil.Uint64 `json:"gasUsed"`
	To           *common.Address `json:"to,omitempty"`
	Input        hexutil.Bytes   `json:"input"`
	Output       hexutil.Bytes   `json:"output,omitempty"`
	Error        string          `json:"error,omitempty"`
	RevertReason string          `json:"revertReason,omitempty"`
	Calls        []callTrace     `json:"calls,omitempty"`
	Logs         []callLog       `json:"logs,omitempty"`
	Value        *hexutil.Big    `json:"value,omitempty"`
	// Gencodec adds overridden fields at the end
	Type string `json:"type"`
}

// callTracerTest defines a single test to check the call tracer against.
type callTracerTest struct {
	Genesis      *core.Genesis   `json:"genesis"`
	Context      *callContext    `json:"context"`
	Input        string          `json:"input"`
	TracerConfig json.RawMessage `json:"tracerConfig"`
	Result       *callTrace      `json:"result"`
}

// Iterates over all the input-output datasets in the tracer test harness and
// runs the JavaScript tracers against them.
func TestCallTracerLegacy(t *testing.T) {
	testCallTracer("callTracerLegacy", "call_tracer_legacy", t)
}

func TestCallTracerNative(t *testing.T) {
	testCallTracer("callTracer", "call_tracer", t)
}

func TestCallTracerNativeWithLog(t *testing.T) {
	testCallTracer("callTracer", "call_tracer_withLog", t)
}

func testCallTracer(tracerName string, dirPath string, t *testing.T) {
	isLegacy := strings.HasSuffix(dirPath, "_legacy")
	files, err := os.ReadDir(filepath.Join("testdata", dirPath))
	if err != nil {
		t.Fatalf("failed to retrieve tracer test suite: %v", err)
	}
	for _, file := range files {
		if !strings.HasSuffix(file.Name(), ".json") {
			continue
		}
		file := file // capture range variable
		t.Run(camel(strings.TrimSuffix(file.Name(), ".json")), func(t *testing.T) {
			t.Parallel()

			var (
				test = new(callTracerTest)
				tx   = new(types.Transaction)
			)
			// Call tracer test found, read if from disk
			if blob, err := os.ReadFile(filepath.Join("testdata", dirPath, file.Name())); err != nil {
				t.Fatalf("failed to read testcase: %v", err)
			} else if err := json.Unmarshal(blob, test); err != nil {
				t.Fatalf("failed to parse testcase: %v", err)
			}
			if err := tx.UnmarshalBinary(common.FromHex(test.Input)); err != nil {
				t.Fatalf("failed to parse testcase input: %v", err)
			}
			// Configure a blockchain with the given prestate
			var (
				signer    = types.MakeSigner(test.Genesis.Config, new(big.Int).SetUint64(uint64(test.Context.Number)), uint64(test.Context.Time))
				origin, _ = signer.Sender(tx)
				txContext = vm.TxContext{
					Origin:   origin,
					GasPrice: tx.GasPrice(),
				}
				context = vm.BlockContext{
					CanTransfer: core.CanTransfer,
					Transfer:    core.Transfer,
					Coinbase:    test.Context.Miner,
					BlockNumber: new(big.Int).SetUint64(uint64(test.Context.Number)),
					Time:        uint64(test.Context.Time),
					Difficulty:  (*big.Int)(test.Context.Difficulty),
					GasLimit:    uint64(test.Context.GasLimit),
					BaseFee:     test.Genesis.BaseFee,
				}
				triedb, _, statedb = tests.MakePreState(rawdb.NewMemoryDatabase(), test.Genesis.Alloc, false, rawdb.HashScheme)
			)
			triedb.Close()

			tracer, err := tracers.DefaultDirectory.New(tracerName, new(tracers.Context), test.TracerConfig)
			if err != nil {
				t.Fatalf("failed to create call tracer: %v", err)
			}
			evm := vm.NewEVM(context, txContext, statedb, test.Genesis.Config, vm.Config{Tracer: tracer})
			msg, err := core.TransactionToMessage(tx, signer, nil)
			if err != nil {
				t.Fatalf("failed to prepare transaction for tracing: %v", err)
			}
			vmRet, err := core.ApplyMessage(evm, msg, new(core.GasPool).AddGas(tx.Gas()))
			if err != nil {
				t.Fatalf("failed to execute transaction: %v", err)
			}
			// Retrieve the trace result and compare against the expected.
			res, err := tracer.GetResult()
			if err != nil {
				t.Fatalf("failed to retrieve trace result: %v", err)
			}
			// The legacy javascript calltracer marshals json in js, which
			// is not deterministic (as opposed to the golang json encoder).
			if isLegacy {
				// This is a tweak to make it deterministic. Can be removed when
				// we remove the legacy tracer.
				var x callTrace
				json.Unmarshal(res, &x)
				res, _ = json.Marshal(x)
			}
			want, err := json.Marshal(test.Result)
			if err != nil {
				t.Fatalf("failed to marshal test: %v", err)
			}
			if string(want) != string(res) {
				t.Fatalf("trace mismatch\n have: %v\n want: %v\n", string(res), string(want))
			}
			// Sanity check: compare top call's gas used against vm result
			type simpleResult struct {
				GasUsed hexutil.Uint64
			}
			var topCall simpleResult
			if err := json.Unmarshal(res, &topCall); err != nil {
				t.Fatalf("failed to unmarshal top calls gasUsed: %v", err)
			}
			if uint64(topCall.GasUsed) != vmRet.UsedGas {
				t.Fatalf("top call has invalid gasUsed. have: %d want: %d", topCall.GasUsed, vmRet.UsedGas)
			}
		})
	}
}

func BenchmarkTracers(b *testing.B) {
	files, err := os.ReadDir(filepath.Join("testdata", "call_tracer"))
	if err != nil {
		b.Fatalf("failed to retrieve tracer test suite: %v", err)
	}
	for _, file := range files {
		if !strings.HasSuffix(file.Name(), ".json") {
			continue
		}
		file := file // capture range variable
		b.Run(camel(strings.TrimSuffix(file.Name(), ".json")), func(b *testing.B) {
			blob, err := os.ReadFile(filepath.Join("testdata", "call_tracer", file.Name()))
			if err != nil {
				b.Fatalf("failed to read testcase: %v", err)
			}
			test := new(callTracerTest)
			if err := json.Unmarshal(blob, test); err != nil {
				b.Fatalf("failed to parse testcase: %v", err)
			}
			benchTracer("callTracer", test, b)
		})
	}
}

func benchTracer(tracerName string, test *callTracerTest, b *testing.B) {
	// Configure a blockchain with the given prestate
	tx := new(types.Transaction)
	if err := rlp.DecodeBytes(common.FromHex(test.Input), tx); err != nil {
		b.Fatalf("failed to parse testcase input: %v", err)
	}
	signer := types.MakeSigner(test.Genesis.Config, new(big.Int).SetUint64(uint64(test.Context.Number)), uint64(test.Context.Time))
	msg, err := core.TransactionToMessage(tx, signer, nil)
	if err != nil {
		b.Fatalf("failed to prepare transaction for tracing: %v", err)
	}
	origin, _ := signer.Sender(tx)
	txContext := vm.TxContext{
		Origin:   origin,
		GasPrice: tx.GasPrice(),
	}
	context := vm.BlockContext{
		CanTransfer: core.CanTransfer,
		Transfer:    core.Transfer,
		Coinbase:    test.Context.Miner,
		BlockNumber: new(big.Int).SetUint64(uint64(test.Context.Number)),
		Time:        uint64(test.Context.Time),
		Difficulty:  (*big.Int)(test.Context.Difficulty),
		GasLimit:    uint64(test.Context.GasLimit),
	}
	triedb, _, statedb := tests.MakePreState(rawdb.NewMemoryDatabase(), test.Genesis.Alloc, false, rawdb.HashScheme)
	defer triedb.Close()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		tracer, err := tracers.DefaultDirectory.New(tracerName, new(tracers.Context), nil)
		if err != nil {
			b.Fatalf("failed to create call tracer: %v", err)
		}
		evm := vm.NewEVM(context, txContext, statedb, test.Genesis.Config, vm.Config{Tracer: tracer})
		snap := statedb.Snapshot()
		st := core.NewStateTransition(evm, msg, new(core.GasPool).AddGas(tx.Gas()))
		if _, err = st.TransitionDb(); err != nil {
			b.Fatalf("failed to execute transaction: %v", err)
		}
		if _, err = tracer.GetResult(); err != nil {
			b.Fatal(err)
		}
		statedb.RevertToSnapshot(snap)
	}
}

func TestInternals(t *testing.T) {
	var (
		to        = common.HexToAddress("0x00000000000000000000000000000000deadbeef")
		origin    = common.HexToAddress("0x00000000000000000000000000000000feed")
		txContext = vm.TxContext{
			Origin:   origin,
			GasPrice: big.NewInt(1),
		}
		context = vm.BlockContext{
			CanTransfer: core.CanTransfer,
			Transfer:    core.Transfer,
			Coinbase:    common.Address{},
			BlockNumber: new(big.Int).SetUint64(8000000),
			Time:        5,
			Difficulty:  big.NewInt(0x30000),
			GasLimit:    uint64(6000000),
		}
	)
	mkTracer := func(name string, cfg json.RawMessage) tracers.Tracer {
		tr, err := tracers.DefaultDirectory.New(name, nil, cfg)
		if err != nil {
			t.Fatalf("failed to create call tracer: %v", err)
		}
		return tr
	}

	for _, tc := range []struct {
		name   string
		code   []byte
		tracer tracers.Tracer
		want   string
	}{
		{
			// TestZeroValueToNotExitCall tests the calltracer(s) on the following:
			// Tx to A, A calls B with zero value. B does not already exist.
			// Expected: that enter/exit is invoked and the inner call is shown in the result
			name: "ZeroValueToNotExitCall",
			code: []byte{
				byte(vm.PUSH1), 0x0, byte(vm.DUP1), byte(vm.DUP1), byte(vm.DUP1), // in and outs zero
				byte(vm.DUP1), byte(vm.PUSH1), 0xff, byte(vm.GAS), // value=0,address=0xff, gas=GAS
				byte(vm.CALL),
			},
			tracer: mkTracer("callTracer", nil),
			want:   `{"from":"0x000000000000000000000000000000000000feed","gas":"0x13880","gasUsed":"0x54d8","to":"0x00000000000000000000000000000000deadbeef","input":"0x","calls":[{"from":"0x00000000000000000000000000000000deadbeef","gas":"0xe01a","gasUsed":"0x0","to":"0x00000000000000000000000000000000000000ff","input":"0x","value":"0x0","type":"CALL"}],"value":"0x0","type":"CALL"}`,
		},
		{
			name:   "Stack depletion in LOG0",
			code:   []byte{byte(vm.LOG3)},
			tracer: mkTracer("callTracer", json.RawMessage(`{ "withLog": true }`)),
			want:   `{"from":"0x000000000000000000000000000000000000feed","gas":"0x13880","gasUsed":"0x13880","to":"0x00000000000000000000000000000000deadbeef","input":"0x","error":"stack underflow (0 \u003c=\u003e 5)","value":"0x0","type":"CALL"}`,
		},
		{
			name: "Mem expansion in LOG0",
			code: []byte{
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0x0,
				byte(vm.MSTORE),
				byte(vm.PUSH1), 0xff,
				byte(vm.PUSH1), 0x0,
				byte(vm.LOG0),
			},
			tracer: mkTracer("callTracer", json.RawMessage(`{ "withLog": true }`)),
			want:   `{"from":"0x000000000000000000000000000000000000feed","gas":"0x13880","gasUsed":"0x5b9e","to":"0x00000000000000000000000000000000deadbeef","input":"0x","logs":[{"address":"0x00000000000000000000000000000000deadbeef","topics":[],"data":"0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}],"value":"0x0","type":"CALL"}`,
		},
		{
			// Leads to OOM on the prestate tracer
			name: "Prestate-tracer - CREATE2 OOM",
			code: []byte{
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0x0,
				byte(vm.MSTORE),
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH5), 0xff, 0xff, 0xff, 0xff, 0xff,
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0x0,
				byte(vm.CREATE2),
				byte(vm.PUSH1), 0xff,
				byte(vm.PUSH1), 0x0,
				byte(vm.LOG0),
			},
			tracer: mkTracer("prestateTracer", nil),
			want:   `{"0x0000000000000000000000000000000000000000":{"balance":"0x0"},"0x000000000000000000000000000000000000feed":{"balance":"0x1c6bf52647880"},"0x00000000000000000000000000000000deadbeef":{"balance":"0x0","code":"0x6001600052600164ffffffffff60016000f560ff6000a0"}}`,
		},
		{
			// CREATE2 which requires padding memory by prestate tracer
			name: "Prestate-tracer - CREATE2 Memory padding",
			code: []byte{
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0x0,
				byte(vm.MSTORE),
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0xff,
				byte(vm.PUSH1), 0x1,
				byte(vm.PUSH1), 0x0,
				byte(vm.CREATE2),
				byte(vm.PUSH1), 0xff,
				byte(vm.PUSH1), 0x0,
				byte(vm.LOG0),
			},
			tracer: mkTracer("prestateTracer", nil),
			want:   `{"0x0000000000000000000000000000000000000000":{"balance":"0x0"},"0x000000000000000000000000000000000000feed":{"balance":"0x1c6bf52647880"},"0x00000000000000000000000000000000deadbeef":{"balance":"0x0","code":"0x6001600052600160ff60016000f560ff6000a0"},"0x91ff9a805d36f54e3e272e230f3e3f5c1b330804":{"balance":"0x0"}}`,
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			triedb, _, statedb := tests.MakePreState(rawdb.NewMemoryDatabase(),
				core.GenesisAlloc{
					to: core.GenesisAccount{
						Code: tc.code,
					},
					origin: core.GenesisAccount{
						Balance: big.NewInt(500000000000000),
					},
				}, false, rawdb.HashScheme)
			defer triedb.Close()

			evm := vm.NewEVM(context, txContext, statedb, params.MainnetChainConfig, vm.Config{Tracer: tc.tracer})
			msg := &core.Message{
				To:                &to,
				From:              origin,
				Value:             big.NewInt(0),
				GasLimit:          80000,
				GasPrice:          big.NewInt(0),
				GasFeeCap:         big.NewInt(0),
				GasTipCap:         big.NewInt(0),
				SkipAccountChecks: false,
			}
			st := core.NewStateTransition(evm, msg, new(core.GasPool).AddGas(msg.GasLimit))
			if _, err := st.TransitionDb(); err != nil {
				t.Fatalf("test %v: failed to execute transaction: %v", tc.name, err)
			}
			// Retrieve the trace result and compare against the expected
			res, err := tc.tracer.GetResult()
			if err != nil {
				t.Fatalf("test %v: failed to retrieve trace result: %v", tc.name, err)
			}
			if string(res) != tc.want {
				t.Errorf("test %v: trace mismatch\n have: %v\n want: %v\n", tc.name, string(res), tc.want)
			}
		})
	}
}
