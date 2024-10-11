package types

import (
	_ "embed"
	"math/rand"

	wasmvm "github.com/CosmWasm/wasmvm/v2"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

//go:embed testdata/reflect.wasm
var reflectWasmCode []byte

func GenesisFixture(mutators ...func(*GenesisState)) GenesisState {
	const (
		numCodes     = 2
		numContracts = 2
		numSequences = 2
		numMsg       = 3
	)

	fixture := GenesisState{
		Params:    DefaultParams(),
		Codes:     make([]Code, numCodes),
		Contracts: make([]Contract, numContracts),
		Sequences: make([]Sequence, numSequences),
	}
	for i := 0; i < numCodes; i++ {
		fixture.Codes[i] = CodeFixture()
	}
	for i := 0; i < numContracts; i++ {
		fixture.Contracts[i] = ContractFixture()
	}
	for i := 0; i < numSequences; i++ {
		fixture.Sequences[i] = Sequence{
			IDKey: randBytes(5),
			Value: uint64(i),
		}
	}

	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

func randBytes(n int) []byte {
	r := make([]byte, n)
	rand.Read(r) //nolint:staticcheck
	return r
}

func CodeFixture(mutators ...func(*Code)) Code {
	fixture := Code{
		CodeID:    1,
		CodeInfo:  CodeInfoFixture(WithSHA256CodeHash(reflectWasmCode)),
		CodeBytes: reflectWasmCode,
	}

	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

func CodeInfoFixture(mutators ...func(*CodeInfo)) CodeInfo {
	codeHash, err := wasmvm.CreateChecksum(reflectWasmCode)
	if err != nil {
		panic(err)
	}
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	fixture := CodeInfo{
		CodeHash:          codeHash[:],
		Creator:           anyAddress,
		InstantiateConfig: AllowEverybody,
	}
	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

func ContractFixture(mutators ...func(*Contract)) Contract {
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"

	fixture := Contract{
		ContractAddress: anyAddress,
		ContractInfo:    ContractInfoFixture(RandCreatedFields),
		ContractState:   []Model{{Key: []byte("anyKey"), Value: []byte("anyValue")}},
	}
	fixture.ContractCodeHistory = []ContractCodeHistoryEntry{ContractCodeHistoryEntryFixture(func(e *ContractCodeHistoryEntry) {
		e.Updated = fixture.ContractInfo.Created
	})}

	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

func OnlyGenesisFields(info *ContractInfo) {
	info.Created = nil
}

func RandCreatedFields(info *ContractInfo) {
	info.Created = &AbsoluteTxPosition{BlockHeight: rand.Uint64(), TxIndex: rand.Uint64()}
}

func ContractInfoFixture(mutators ...func(*ContractInfo)) ContractInfo {
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"

	fixture := ContractInfo{
		CodeID:  1,
		Creator: anyAddress,
		Label:   "any",
		Created: &AbsoluteTxPosition{BlockHeight: 1, TxIndex: 1},
	}

	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

// ContractCodeHistoryEntryFixture test fixture
func ContractCodeHistoryEntryFixture(mutators ...func(*ContractCodeHistoryEntry)) ContractCodeHistoryEntry {
	fixture := ContractCodeHistoryEntry{
		Operation: ContractCodeHistoryOperationTypeInit,
		CodeID:    1,
		Updated:   ContractInfoFixture().Created,
		Msg:       []byte(`{"foo":"bar"}`),
	}
	for _, m := range mutators {
		m(&fixture)
	}
	return fixture
}

func WithSHA256CodeHash(wasmCode []byte) func(info *CodeInfo) {
	return func(info *CodeInfo) {
		codeHash, err := wasmvm.CreateChecksum(wasmCode)
		if err != nil {
			panic(err)
		}
		info.CodeHash = codeHash[:]
	}
}

func MsgStoreCodeFixture(mutators ...func(*MsgStoreCode)) *MsgStoreCode {
	wasmIdent := []byte("\x00\x61\x73\x6D")
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	r := &MsgStoreCode{
		Sender:                anyAddress,
		WASMByteCode:          wasmIdent,
		InstantiatePermission: &AllowEverybody,
	}
	for _, m := range mutators {
		m(r)
	}
	return r
}

func MsgInstantiateContractFixture(mutators ...func(*MsgInstantiateContract)) *MsgInstantiateContract {
	const anyAddress = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
	r := &MsgInstantiateContract{
		Sender: anyAddress,
		Admin:  anyAddress,
		CodeID: 1,
		Label:  "testing",
		Msg:    []byte(`{"foo":"bar"}`),
		Funds: sdk.Coins{{
			Denom:  "stake",
			Amount: sdkmath.NewInt(1),
		}},
	}
	for _, m := range mutators {
		m(r)
	}
	return r
}

func MsgExecuteContractFixture(mutators ...func(*MsgExecuteContract)) *MsgExecuteContract {
	const (
		anyAddress           = "cosmos1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqs2m6sx4"
		firstContractAddress = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"
	)
	r := &MsgExecuteContract{
		Sender:   anyAddress,
		Contract: firstContractAddress,
		Msg:      []byte(`{"do":"something"}`),
		Funds: sdk.Coins{{
			Denom:  "stake",
			Amount: sdkmath.NewInt(1),
		}},
	}
	for _, m := range mutators {
		m(r)
	}
	return r
}
