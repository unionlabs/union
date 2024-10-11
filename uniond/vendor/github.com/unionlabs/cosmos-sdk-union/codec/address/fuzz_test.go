package address

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/address"

	sdkAddress "github.com/cosmos/cosmos-sdk/types/address"
)

func FuzzCachedAddressCodec(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}

	addresses, err := generateAddresses(2)
	require.NoError(f, err)

	for _, addr := range addresses {
		f.Add(addr)
	}
	cdc, err := NewCachedBech32Codec("cosmos", cacheOptions)
	require.NoError(f, err)

	f.Fuzz(func(t *testing.T, addr []byte) {
		checkAddress(t, addr, cdc)
	})
}

func FuzzAddressCodec(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}
	addresses, err := generateAddresses(2)
	require.NoError(f, err)

	for _, addr := range addresses {
		f.Add(addr)
	}

	cdc := Bech32Codec{Bech32Prefix: "cosmos"}

	f.Fuzz(func(t *testing.T, addr []byte) {
		checkAddress(t, addr, cdc)
	})
}

func checkAddress(t *testing.T, addr []byte, cdc address.Codec) {
	t.Helper()
	if len(addr) > sdkAddress.MaxAddrLen {
		return
	}
	strAddr, err := cdc.BytesToString(addr)
	if err != nil {
		t.Fatal(err)
	}
	b, err := cdc.StringToBytes(strAddr)
	if err != nil {
		if !errors.Is(errEmptyAddress, err) {
			t.Fatal(err)
		}
	}
	require.Equal(t, len(addr), len(b))
}
