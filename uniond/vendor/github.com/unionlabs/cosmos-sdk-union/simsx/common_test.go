package simsx

import (
	"context"
	"math/rand"

	"github.com/cosmos/gogoproto/proto"

	coretransaction "cosmossdk.io/core/transaction"
	"cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/std"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// SimAccountFixture testing only
func SimAccountFixture(mutators ...func(account *SimAccount)) SimAccount {
	r := rand.New(rand.NewSource(1))
	acc := SimAccount{
		Account: simtypes.RandomAccounts(r, 1)[0],
	}
	acc.liquidBalance = NewSimsAccountBalance(&acc, r, sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1_000_000_000)))
	for _, mutator := range mutators {
		mutator(&acc)
	}
	return acc
}

// MemoryAccountSource testing only
func MemoryAccountSource(srcs ...SimAccount) AccountSourceFn {
	accs := make(map[string]FakeAccountI, len(srcs))
	for _, src := range srcs {
		accs[src.AddressBech32] = FakeAccountI{SimAccount: src, id: 1, seq: 2}
	}
	return func(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
		return accs[addr.String()]
	}
}

// testing only
func txConfig() client.TxConfig {
	ir := must(codectypes.NewInterfaceRegistryWithOptions(codectypes.InterfaceRegistryOptions{
		ProtoFiles: proto.HybridResolver,
		SigningOptions: signing.Options{
			AddressCodec:          address.NewBech32Codec("cosmos"),
			ValidatorAddressCodec: address.NewBech32Codec("cosmosvaloper"),
		},
	}))
	std.RegisterInterfaces(ir)
	ir.RegisterImplementations((*coretransaction.Msg)(nil), &testdata.TestMsg{})
	protoCodec := codec.NewProtoCodec(ir)
	signingCtx := protoCodec.InterfaceRegistry().SigningContext()
	return tx.NewTxConfig(protoCodec, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), tx.DefaultSignModes)
}

var _ AppEntrypoint = SimDeliverFn(nil)

type (
	AppEntrypointFn = SimDeliverFn
	SimDeliverFn    func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error)
)

func (m SimDeliverFn) SimDeliver(txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
	return m(txEncoder, tx)
}

var _ AccountSource = AccountSourceFn(nil)

type AccountSourceFn func(ctx context.Context, addr sdk.AccAddress) sdk.AccountI

func (a AccountSourceFn) GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
	return a(ctx, addr)
}

var _ sdk.AccountI = &FakeAccountI{}

type FakeAccountI struct {
	SimAccount
	id, seq uint64
}

func (m FakeAccountI) GetAddress() sdk.AccAddress {
	return m.Address
}

func (m FakeAccountI) GetPubKey() cryptotypes.PubKey {
	return m.PubKey
}

func (m FakeAccountI) GetAccountNumber() uint64 {
	return m.id
}

func (m FakeAccountI) GetSequence() uint64 {
	return m.seq
}

func (m FakeAccountI) Reset() {
	panic("implement me")
}

func (m FakeAccountI) String() string {
	panic("implement me")
}

func (m FakeAccountI) ProtoMessage() {
	panic("implement me")
}

func (m FakeAccountI) SetAddress(address sdk.AccAddress) error {
	panic("implement me")
}

func (m FakeAccountI) SetPubKey(key cryptotypes.PubKey) error {
	panic("implement me")
}

func (m FakeAccountI) SetAccountNumber(u uint64) error {
	panic("implement me")
}

func (m FakeAccountI) SetSequence(u uint64) error {
	panic("implement me")
}

var _ AccountSourceX = &MockAccountSourceX{}

// MockAccountSourceX mock impl for testing only
type MockAccountSourceX struct {
	GetAccountFn       func(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	GetModuleAddressFn func(moduleName string) sdk.AccAddress
}

func (m MockAccountSourceX) GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
	if m.GetAccountFn == nil {
		panic("not expected to be called")
	}
	return m.GetAccountFn(ctx, addr)
}

func (m MockAccountSourceX) GetModuleAddress(moduleName string) sdk.AccAddress {
	if m.GetModuleAddressFn == nil {
		panic("not expected to be called")
	}
	return m.GetModuleAddressFn(moduleName)
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}
