package types

import (
	"math"
	"strings"
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/stretchr/testify/assert"

	storetypes "cosmossdk.io/store/types"
)

func TestSetupContractCost(t *testing.T) {
	specs := map[string]struct {
		srcLen    int
		srcConfig WasmGasRegisterConfig
		pinned    bool
		exp       storetypes.Gas
		expPanic  bool
	}{
		"small msg - pinned": {
			srcLen:    1,
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + DefaultContractMessageDataCost,
		},
		"big msg - pinned": {
			srcLen:    math.MaxUint32,
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + DefaultContractMessageDataCost*math.MaxUint32,
		},
		"empty msg - pinned": {
			srcLen:    0,
			pinned:    true,
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCostDiscount,
		},
		"small msg - unpinned": {
			srcLen:    1,
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultContractMessageDataCost + DefaultInstanceCost,
		},
		"big msg - unpinned": {
			srcLen:    math.MaxUint32,
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultContractMessageDataCost*math.MaxUint32 + DefaultInstanceCost,
		},
		"empty msg - unpinned": {
			srcLen:    0,
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost,
		},
		"negative len": {
			srcLen:    -1,
			srcConfig: DefaultGasRegisterConfig(),
			expPanic:  true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if spec.expPanic {
				assert.Panics(t, func() {
					NewWasmGasRegister(spec.srcConfig).SetupContractCost(spec.pinned, spec.srcLen)
				})
				return
			}
			gotGas := NewWasmGasRegister(spec.srcConfig).SetupContractCost(spec.pinned, spec.srcLen)
			assert.Equal(t, spec.exp, gotGas)
		})
	}
}

func TestReplyCost(t *testing.T) {
	specs := map[string]struct {
		src       wasmvmtypes.Reply
		srcConfig WasmGasRegisterConfig
		pinned    bool
		exp       storetypes.Gas
		expPanic  bool
	}{
		"submessage reply with events and data - pinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: "myKey", Value: "myData"}}},
						},
						Data: []byte{0x1},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + 3*DefaultEventAttributeDataCost + DefaultPerAttributeCost + DefaultContractMessageDataCost, // 3 == len("foo")
		},
		"submessage reply with events - pinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: "myKey", Value: "myData"}}},
						},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + 3*DefaultEventAttributeDataCost + DefaultPerAttributeCost, // 3 == len("foo")
		},
		"submessage reply with events exceeds free tier - pinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: strings.Repeat("x", DefaultEventAttributeDataFreeTier), Value: "myData"}}},
						},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + (3+6)*DefaultEventAttributeDataCost + DefaultPerAttributeCost, // 3 == len("foo"), 6 == len("myData")
		},
		"submessage reply error - pinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Err: "foo",
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			pinned:    true,
			exp:       DefaultInstanceCostDiscount + 3*DefaultContractMessageDataCost,
		},
		"submessage reply with events and data - unpinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: "myKey", Value: "myData"}}},
						},
						Data: []byte{0x1},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost + 3*DefaultEventAttributeDataCost + DefaultPerAttributeCost + DefaultContractMessageDataCost,
		},
		"submessage reply with events - unpinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: "myKey", Value: "myData"}}},
						},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost + 3*DefaultEventAttributeDataCost + DefaultPerAttributeCost,
		},
		"submessage reply with events exceeds free tier - unpinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: []wasmvmtypes.Event{
							{Type: "foo", Attributes: []wasmvmtypes.EventAttribute{{Key: strings.Repeat("x", DefaultEventAttributeDataFreeTier), Value: "myData"}}},
						},
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost + (3+6)*DefaultEventAttributeDataCost + DefaultPerAttributeCost, // 3 == len("foo"), 6 == len("myData")
		},
		"submessage reply error - unpinned": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Err: "foo",
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost + 3*DefaultContractMessageDataCost,
		},
		"submessage reply with empty events": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{
						Events: make([]wasmvmtypes.Event, 10),
					},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost,
		},
		"submessage reply with events unset": {
			src: wasmvmtypes.Reply{
				Result: wasmvmtypes.SubMsgResult{
					Ok: &wasmvmtypes.SubMsgResponse{},
				},
			},
			srcConfig: DefaultGasRegisterConfig(),
			exp:       DefaultInstanceCost,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if spec.expPanic {
				assert.Panics(t, func() {
					NewWasmGasRegister(spec.srcConfig).ReplyCosts(spec.pinned, spec.src)
				})
				return
			}
			gotGas := NewWasmGasRegister(spec.srcConfig).ReplyCosts(spec.pinned, spec.src)
			assert.Equal(t, spec.exp, gotGas)
		})
	}
}

func TestEventCosts(t *testing.T) {
	// most cases are covered in TestReplyCost already. This ensures some edge cases
	specs := map[string]struct {
		srcAttrs  []wasmvmtypes.EventAttribute
		srcEvents wasmvmtypes.Array[wasmvmtypes.Event]
		expGas    storetypes.Gas
	}{
		"empty events": {
			srcEvents: make([]wasmvmtypes.Event, 1),
			expGas:    DefaultPerCustomEventCost,
		},
		"empty attributes": {
			srcAttrs: make([]wasmvmtypes.EventAttribute, 1),
			expGas:   DefaultPerAttributeCost,
		},
		"both nil": {
			expGas: 0,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotGas := NewDefaultWasmGasRegister().EventCosts(spec.srcAttrs, spec.srcEvents)
			assert.Equal(t, spec.expGas, gotGas)
		})
	}
}

func TestToWasmVMGasConversion(t *testing.T) {
	specs := map[string]struct {
		src       storetypes.Gas
		srcConfig WasmGasRegisterConfig
		exp       uint64
		expPanic  bool
	}{
		"0": {
			src:       0,
			exp:       0,
			srcConfig: DefaultGasRegisterConfig(),
		},
		"max": {
			srcConfig: WasmGasRegisterConfig{
				GasMultiplier: 1,
			},
			src: math.MaxUint64,
			exp: math.MaxUint64,
		},
		"overflow": {
			srcConfig: WasmGasRegisterConfig{
				GasMultiplier: 2,
			},
			src:      math.MaxUint64,
			expPanic: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if spec.expPanic {
				assert.Panics(t, func() {
					r := NewWasmGasRegister(spec.srcConfig)
					_ = r.ToWasmVMGas(spec.src)
				})
				return
			}
			r := NewWasmGasRegister(spec.srcConfig)
			got := r.ToWasmVMGas(spec.src)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestFromWasmVMGasConversion(t *testing.T) {
	specs := map[string]struct {
		src       uint64
		exp       storetypes.Gas
		srcConfig WasmGasRegisterConfig
		expPanic  bool
	}{
		"0": {
			src:       0,
			exp:       0,
			srcConfig: DefaultGasRegisterConfig(),
		},
		"max": {
			srcConfig: WasmGasRegisterConfig{
				GasMultiplier: 1,
			},
			src: math.MaxUint64,
			exp: math.MaxUint64,
		},
		"missconfigured": {
			srcConfig: WasmGasRegisterConfig{
				GasMultiplier: 0,
			},
			src:      1,
			expPanic: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if spec.expPanic {
				assert.Panics(t, func() {
					r := NewWasmGasRegister(spec.srcConfig)
					_ = r.FromWasmVMGas(spec.src)
				})
				return
			}
			r := NewWasmGasRegister(spec.srcConfig)
			got := r.FromWasmVMGas(spec.src)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestUncompressCosts(t *testing.T) {
	specs := map[string]struct {
		lenIn    int
		exp      storetypes.Gas
		expPanic bool
	}{
		"0": {
			exp: 0,
		},
		"even": {
			lenIn: 100,
			exp:   15,
		},
		"round down when uneven": {
			lenIn: 19,
			exp:   2,
		},
		"max len": {
			lenIn: MaxWasmSize,
			exp:   122880,
		},
		"invalid len": {
			lenIn:    -1,
			expPanic: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			if spec.expPanic {
				assert.Panics(t, func() { NewDefaultWasmGasRegister().UncompressCosts(spec.lenIn) })
				return
			}
			got := NewDefaultWasmGasRegister().UncompressCosts(spec.lenIn)
			assert.Equal(t, spec.exp, got)
		})
	}
}
