package keeper

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestDontBindPortNonIBCContract(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	example := InstantiateHackatomExampleContract(t, ctx, keepers) // ensure we bound the port
	_, _, err := keepers.IBCKeeper.PortKeeper.LookupModuleByPort(ctx, keepers.WasmKeeper.GetContractInfo(ctx, example.Contract).IBCPortID)
	require.Error(t, err)
}

func TestBindingPortForIBCContractOnInstantiate(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	example := InstantiateIBCReflectContract(t, ctx, keepers) // ensure we bound the port
	owner, _, err := keepers.IBCKeeper.PortKeeper.LookupModuleByPort(ctx, keepers.WasmKeeper.GetContractInfo(ctx, example.Contract).IBCPortID)
	require.NoError(t, err)
	require.Equal(t, "wasm", owner)

	initMsgBz, err := json.Marshal(IBCReflectInitMsg{
		ReflectCodeID: example.ReflectCodeID,
	})
	require.NoError(t, err)

	// create a second contract should give yet another portID (and different address)
	creator := RandomAccountAddress(t)
	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, example.CodeID, creator, nil, initMsgBz, "ibc-reflect-2", nil)
	require.NoError(t, err)
	require.NotEqual(t, example.Contract, addr)

	portID2 := PortIDForContract(addr)
	owner, _, err = keepers.IBCKeeper.PortKeeper.LookupModuleByPort(ctx, portID2)
	require.NoError(t, err)
	require.Equal(t, "wasm", owner)
}

func TestContractFromPortID(t *testing.T) {
	contractAddr := BuildContractAddressClassic(1, 100)
	specs := map[string]struct {
		srcPort string
		expAddr sdk.AccAddress
		expErr  bool
	}{
		"all good": {
			srcPort: fmt.Sprintf("wasm.%s", contractAddr.String()),
			expAddr: contractAddr,
		},
		"without prefix": {
			srcPort: contractAddr.String(),
			expErr:  true,
		},
		"invalid prefix": {
			srcPort: fmt.Sprintf("wasmx.%s", contractAddr.String()),
			expErr:  true,
		},
		"without separator char": {
			srcPort: fmt.Sprintf("wasm%s", contractAddr.String()),
			expErr:  true,
		},
		"invalid account": {
			srcPort: "wasm.foobar",
			expErr:  true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotAddr, gotErr := ContractFromPortID(spec.srcPort)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expAddr, gotAddr)
		})
	}
}
