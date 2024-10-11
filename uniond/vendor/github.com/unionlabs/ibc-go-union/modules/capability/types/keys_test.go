package types_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/modules/capability/types"
)

func TestRevCapabilityKey(t *testing.T) {
	expected := []byte("bank/rev/send")
	require.Equal(t, expected, types.RevCapabilityKey("bank", "send"))
}

func TestFwdCapabilityKey(t *testing.T) {
	capability := types.NewCapability(23)
	expected := []byte(fmt.Sprintf("bank/fwd/%#016p", capability))
	require.Equal(t, expected, types.FwdCapabilityKey("bank", capability))
}

func TestIndexToKey(t *testing.T) {
	require.Equal(t, []byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xc, 0x5a}, types.IndexToKey(3162))
}

func TestIndexFromKey(t *testing.T) {
	require.Equal(t, uint64(3162), types.IndexFromKey([]byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xc, 0x5a}))
}
