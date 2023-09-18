package types_test

import (
	"fmt"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	pk1      = ed25519.GenPrivKey().PubKey()
	pk1Any   *codectypes.Any
	pk2      = ed25519.GenPrivKey().PubKey()
	pk3      = ed25519.GenPrivKey().PubKey()
	addr1, _ = sdk.Bech32ifyAddressBytes(sdk.Bech32PrefixAccAddr, pk1.Address().Bytes())
	addr2, _ = sdk.Bech32ifyAddressBytes(sdk.Bech32PrefixAccAddr, pk2.Address().Bytes())
	addr3, _ = sdk.Bech32ifyAddressBytes(sdk.Bech32PrefixAccAddr, pk3.Address().Bytes())
	valAddr1 = sdk.ValAddress(pk1.Address())
	valAddr2 = sdk.ValAddress(pk2.Address())
	valAddr3 = sdk.ValAddress(pk3.Address())

	emptyAddr   sdk.ValAddress
	emptyPubkey cryptotypes.PubKey
)

func init() {
	var err error
	pk1Any, err = codectypes.NewAnyWithValue(pk1)
	if err != nil {
		panic(fmt.Sprintf("Can't pack pk1 %t as Any", pk1))
	}
}
