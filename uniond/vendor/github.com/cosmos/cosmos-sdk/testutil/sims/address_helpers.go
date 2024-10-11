package sims

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"strconv"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const mintModuleName = "mint"

type GenerateAccountStrategy func(int) []sdk.AccAddress

// AddTestAddrsFromPubKeys adds the addresses into the SimApp providing only the public keys.
func AddTestAddrsFromPubKeys(bankKeeper BankKeeper, stakingKeeper StakingKeeper, ctx sdk.Context, pubKeys []cryptotypes.PubKey, accAmt math.Int) {
	bondDenom, err := stakingKeeper.BondDenom(ctx)
	if err != nil {
		panic(err)
	}
	initCoins := sdk.NewCoins(sdk.NewCoin(bondDenom, accAmt))

	for _, pk := range pubKeys {
		initAccountWithCoins(bankKeeper, ctx, sdk.AccAddress(pk.Address()), initCoins)
	}
}

// AddTestAddrs constructs and returns accNum amount of accounts with an
// initial balance of accAmt in random order
func AddTestAddrs(bankKeeper BankKeeper, stakingKeeper StakingKeeper, ctx sdk.Context, accNum int, accAmt math.Int) []sdk.AccAddress {
	return addTestAddrs(bankKeeper, stakingKeeper, ctx, accNum, accAmt, CreateRandomAccounts)
}

// AddTestAddrsIncremental constructs and returns accNum amount of accounts with an initial balance of accAmt in random order
func AddTestAddrsIncremental(bankKeeper BankKeeper, stakingKeeper StakingKeeper, ctx sdk.Context, accNum int, accAmt math.Int) []sdk.AccAddress {
	return addTestAddrs(bankKeeper, stakingKeeper, ctx, accNum, accAmt, CreateIncrementalAccounts)
}

func addTestAddrs(bankKeeper BankKeeper, stakingKeeper StakingKeeper, ctx sdk.Context, accNum int, accAmt math.Int, strategy GenerateAccountStrategy) []sdk.AccAddress {
	testAddrs := strategy(accNum)
	bondDenom, err := stakingKeeper.BondDenom(ctx)
	if err != nil {
		panic(err)
	}
	initCoins := sdk.NewCoins(sdk.NewCoin(bondDenom, accAmt))

	for _, addr := range testAddrs {
		initAccountWithCoins(bankKeeper, ctx, addr, initCoins)
	}

	return testAddrs
}

func initAccountWithCoins(bankKeeper BankKeeper, ctx sdk.Context, addr sdk.AccAddress, coins sdk.Coins) {
	if err := bankKeeper.MintCoins(ctx, mintModuleName, coins); err != nil {
		panic(err)
	}

	if err := bankKeeper.SendCoinsFromModuleToAccount(ctx, mintModuleName, addr, coins); err != nil {
		panic(err)
	}
}

// CreateIncrementalAccounts is a strategy used by addTestAddrs() in order to generated addresses in ascending order.
func CreateIncrementalAccounts(accNum int) []sdk.AccAddress {
	var addresses []sdk.AccAddress
	var buffer bytes.Buffer

	// start at 100 so we can make up to 999 test addresses with valid test addresses
	for i := 100; i < (accNum + 100); i++ {
		numString := strconv.Itoa(i)
		buffer.WriteString("A58856F0FD53BF058B4909A21AEC019107BA6") // base address string

		buffer.WriteString(numString) // adding on final two digits to make addresses unique
		res, _ := sdk.AccAddressFromHexUnsafe(buffer.String())
		bech := res.String()
		addr, _ := TestAddr(buffer.String(), bech)

		addresses = append(addresses, addr)
		buffer.Reset()
	}

	return addresses
}

// CreateRandomAccounts is a strategy used by addTestAddrs() in order to generated addresses in random order.
func CreateRandomAccounts(accNum int) []sdk.AccAddress {
	testAddrs := make([]sdk.AccAddress, accNum)
	for i := 0; i < accNum; i++ {
		pk := ed25519.GenPrivKey().PubKey()
		testAddrs[i] = sdk.AccAddress(pk.Address())
	}

	return testAddrs
}

func TestAddr(addr, bech string) (sdk.AccAddress, error) {
	res, err := sdk.AccAddressFromHexUnsafe(addr)
	if err != nil {
		return nil, err
	}
	bechexpected := res.String()
	if bech != bechexpected {
		return nil, fmt.Errorf("bech encoding doesn't match reference")
	}

	bechres, err := sdk.AccAddressFromBech32(bech)
	if err != nil {
		return nil, err
	}
	if !bytes.Equal(bechres, res) {
		return nil, err
	}

	return res, nil
}

// ConvertAddrsToValAddrs converts the provided addresses to ValAddress.
func ConvertAddrsToValAddrs(addrs []sdk.AccAddress) []sdk.ValAddress {
	valAddrs := make([]sdk.ValAddress, len(addrs))

	for i, addr := range addrs {
		valAddrs[i] = sdk.ValAddress(addr)
	}

	return valAddrs
}

// CreateTestPubKeys returns a total of numPubKeys public keys in ascending order.
func CreateTestPubKeys(numPubKeys int) []cryptotypes.PubKey {
	var publicKeys []cryptotypes.PubKey
	var buffer bytes.Buffer

	// start at 10 to avoid changing 1 to 01, 2 to 02, etc
	for i := 100; i < (numPubKeys + 100); i++ {
		numString := strconv.Itoa(i)
		buffer.WriteString("0B485CFC0EECC619440448436F8FC9DF40566F2369E72400281454CB552AF") // base pubkey string
		buffer.WriteString(numString)                                                       // adding on final two digits to make pubkeys unique
		publicKeys = append(publicKeys, NewPubKeyFromHex(buffer.String()))
		buffer.Reset()
	}

	return publicKeys
}

// NewPubKeyFromHex returns a PubKey from a hex string.
func NewPubKeyFromHex(pk string) (res cryptotypes.PubKey) {
	pkBytes, err := hex.DecodeString(pk)
	if err != nil {
		panic(err)
	}
	if len(pkBytes) != ed25519.PubKeySize {
		panic(errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "invalid pubkey size"))
	}
	return &ed25519.PubKey{Key: pkBytes}
}
