// Copyright 2019 The go-ethereum Authors
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

package bind_test

import (
	"context"
	"errors"
	"math/big"
	"reflect"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/stretchr/testify/assert"
)

func mockSign(addr common.Address, tx *types.Transaction) (*types.Transaction, error) { return tx, nil }

type mockTransactor struct {
	baseFee                *big.Int
	gasTipCap              *big.Int
	gasPrice               *big.Int
	suggestGasTipCapCalled bool
	suggestGasPriceCalled  bool
}

func (mt *mockTransactor) HeaderByNumber(ctx context.Context, number *big.Int) (*types.Header, error) {
	return &types.Header{BaseFee: mt.baseFee}, nil
}

func (mt *mockTransactor) PendingCodeAt(ctx context.Context, account common.Address) ([]byte, error) {
	return []byte{1}, nil
}

func (mt *mockTransactor) PendingNonceAt(ctx context.Context, account common.Address) (uint64, error) {
	return 0, nil
}

func (mt *mockTransactor) SuggestGasPrice(ctx context.Context) (*big.Int, error) {
	mt.suggestGasPriceCalled = true
	return mt.gasPrice, nil
}

func (mt *mockTransactor) SuggestGasTipCap(ctx context.Context) (*big.Int, error) {
	mt.suggestGasTipCapCalled = true
	return mt.gasTipCap, nil
}

func (mt *mockTransactor) EstimateGas(ctx context.Context, call ethereum.CallMsg) (gas uint64, err error) {
	return 0, nil
}

func (mt *mockTransactor) SendTransaction(ctx context.Context, tx *types.Transaction) error {
	return nil
}

type mockCaller struct {
	codeAtBlockNumber       *big.Int
	callContractBlockNumber *big.Int
	callContractBytes       []byte
	callContractErr         error
	codeAtBytes             []byte
	codeAtErr               error
}

func (mc *mockCaller) CodeAt(ctx context.Context, contract common.Address, blockNumber *big.Int) ([]byte, error) {
	mc.codeAtBlockNumber = blockNumber
	return mc.codeAtBytes, mc.codeAtErr
}

func (mc *mockCaller) CallContract(ctx context.Context, call ethereum.CallMsg, blockNumber *big.Int) ([]byte, error) {
	mc.callContractBlockNumber = blockNumber
	return mc.callContractBytes, mc.callContractErr
}

type mockPendingCaller struct {
	*mockCaller
	pendingCodeAtBytes        []byte
	pendingCodeAtErr          error
	pendingCodeAtCalled       bool
	pendingCallContractCalled bool
	pendingCallContractBytes  []byte
	pendingCallContractErr    error
}

func (mc *mockPendingCaller) PendingCodeAt(ctx context.Context, contract common.Address) ([]byte, error) {
	mc.pendingCodeAtCalled = true
	return mc.pendingCodeAtBytes, mc.pendingCodeAtErr
}

func (mc *mockPendingCaller) PendingCallContract(ctx context.Context, call ethereum.CallMsg) ([]byte, error) {
	mc.pendingCallContractCalled = true
	return mc.pendingCallContractBytes, mc.pendingCallContractErr
}

type mockBlockHashCaller struct {
	*mockCaller
	codeAtHashBytes          []byte
	codeAtHashErr            error
	codeAtHashCalled         bool
	callContractAtHashCalled bool
	callContractAtHashBytes  []byte
	callContractAtHashErr    error
}

func (mc *mockBlockHashCaller) CodeAtHash(ctx context.Context, contract common.Address, hash common.Hash) ([]byte, error) {
	mc.codeAtHashCalled = true
	return mc.codeAtHashBytes, mc.codeAtHashErr
}

func (mc *mockBlockHashCaller) CallContractAtHash(ctx context.Context, call ethereum.CallMsg, hash common.Hash) ([]byte, error) {
	mc.callContractAtHashCalled = true
	return mc.callContractAtHashBytes, mc.callContractAtHashErr
}

func TestPassingBlockNumber(t *testing.T) {
	mc := &mockPendingCaller{
		mockCaller: &mockCaller{
			codeAtBytes: []byte{1, 2, 3},
		},
	}

	bc := bind.NewBoundContract(common.HexToAddress("0x0"), abi.ABI{
		Methods: map[string]abi.Method{
			"something": {
				Name:    "something",
				Outputs: abi.Arguments{},
			},
		},
	}, mc, nil, nil)

	blockNumber := big.NewInt(42)

	bc.Call(&bind.CallOpts{BlockNumber: blockNumber}, nil, "something")

	if mc.callContractBlockNumber != blockNumber {
		t.Fatalf("CallContract() was not passed the block number")
	}

	if mc.codeAtBlockNumber != blockNumber {
		t.Fatalf("CodeAt() was not passed the block number")
	}

	bc.Call(&bind.CallOpts{}, nil, "something")

	if mc.callContractBlockNumber != nil {
		t.Fatalf("CallContract() was passed a block number when it should not have been")
	}

	if mc.codeAtBlockNumber != nil {
		t.Fatalf("CodeAt() was passed a block number when it should not have been")
	}

	bc.Call(&bind.CallOpts{BlockNumber: blockNumber, Pending: true}, nil, "something")

	if !mc.pendingCallContractCalled {
		t.Fatalf("CallContract() was not passed the block number")
	}

	if !mc.pendingCodeAtCalled {
		t.Fatalf("CodeAt() was not passed the block number")
	}
}

const hexData = "0x000000000000000000000000376c47978271565f56deb45495afa69e59c16ab200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158"

func TestUnpackIndexedStringTyLogIntoMap(t *testing.T) {
	hash := crypto.Keccak256Hash([]byte("testName"))
	topics := []common.Hash{
		crypto.Keccak256Hash([]byte("received(string,address,uint256,bytes)")),
		hash,
	}
	mockLog := newMockLog(topics, common.HexToHash("0x0"))

	abiString := `[{"anonymous":false,"inputs":[{"indexed":true,"name":"name","type":"string"},{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	expectedReceivedMap := map[string]interface{}{
		"name":   hash,
		"sender": common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount": big.NewInt(1),
		"memo":   []byte{88},
	}
	unpackAndCheck(t, bc, expectedReceivedMap, mockLog)
}

func TestUnpackAnonymousLogIntoMap(t *testing.T) {
	mockLog := newMockLog(nil, common.HexToHash("0x0"))

	abiString := `[{"anonymous":false,"inputs":[{"indexed":false,"name":"amount","type":"uint256"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	var received map[string]interface{}
	err := bc.UnpackLogIntoMap(received, "received", mockLog)
	if err == nil {
		t.Error("unpacking anonymous event is not supported")
	}
	if err.Error() != "no event signature" {
		t.Errorf("expected error 'no event signature', got '%s'", err)
	}
}

func TestUnpackIndexedSliceTyLogIntoMap(t *testing.T) {
	sliceBytes, err := rlp.EncodeToBytes([]string{"name1", "name2", "name3", "name4"})
	if err != nil {
		t.Fatal(err)
	}
	hash := crypto.Keccak256Hash(sliceBytes)
	topics := []common.Hash{
		crypto.Keccak256Hash([]byte("received(string[],address,uint256,bytes)")),
		hash,
	}
	mockLog := newMockLog(topics, common.HexToHash("0x0"))

	abiString := `[{"anonymous":false,"inputs":[{"indexed":true,"name":"names","type":"string[]"},{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	expectedReceivedMap := map[string]interface{}{
		"names":  hash,
		"sender": common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount": big.NewInt(1),
		"memo":   []byte{88},
	}
	unpackAndCheck(t, bc, expectedReceivedMap, mockLog)
}

func TestUnpackIndexedArrayTyLogIntoMap(t *testing.T) {
	arrBytes, err := rlp.EncodeToBytes([2]common.Address{common.HexToAddress("0x0"), common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2")})
	if err != nil {
		t.Fatal(err)
	}
	hash := crypto.Keccak256Hash(arrBytes)
	topics := []common.Hash{
		crypto.Keccak256Hash([]byte("received(address[2],address,uint256,bytes)")),
		hash,
	}
	mockLog := newMockLog(topics, common.HexToHash("0x0"))

	abiString := `[{"anonymous":false,"inputs":[{"indexed":true,"name":"addresses","type":"address[2]"},{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	expectedReceivedMap := map[string]interface{}{
		"addresses": hash,
		"sender":    common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount":    big.NewInt(1),
		"memo":      []byte{88},
	}
	unpackAndCheck(t, bc, expectedReceivedMap, mockLog)
}

func TestUnpackIndexedFuncTyLogIntoMap(t *testing.T) {
	mockAddress := common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2")
	addrBytes := mockAddress.Bytes()
	hash := crypto.Keccak256Hash([]byte("mockFunction(address,uint)"))
	functionSelector := hash[:4]
	functionTyBytes := append(addrBytes, functionSelector...)
	var functionTy [24]byte
	copy(functionTy[:], functionTyBytes[0:24])
	topics := []common.Hash{
		crypto.Keccak256Hash([]byte("received(function,address,uint256,bytes)")),
		common.BytesToHash(functionTyBytes),
	}
	mockLog := newMockLog(topics, common.HexToHash("0x5c698f13940a2153440c6d19660878bc90219d9298fdcf37365aa8d88d40fc42"))
	abiString := `[{"anonymous":false,"inputs":[{"indexed":true,"name":"function","type":"function"},{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	expectedReceivedMap := map[string]interface{}{
		"function": functionTy,
		"sender":   common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount":   big.NewInt(1),
		"memo":     []byte{88},
	}
	unpackAndCheck(t, bc, expectedReceivedMap, mockLog)
}

func TestUnpackIndexedBytesTyLogIntoMap(t *testing.T) {
	bytes := []byte{1, 2, 3, 4, 5}
	hash := crypto.Keccak256Hash(bytes)
	topics := []common.Hash{
		crypto.Keccak256Hash([]byte("received(bytes,address,uint256,bytes)")),
		hash,
	}
	mockLog := newMockLog(topics, common.HexToHash("0x5c698f13940a2153440c6d19660878bc90219d9298fdcf37365aa8d88d40fc42"))

	abiString := `[{"anonymous":false,"inputs":[{"indexed":true,"name":"content","type":"bytes"},{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"}]`
	parsedAbi, _ := abi.JSON(strings.NewReader(abiString))
	bc := bind.NewBoundContract(common.HexToAddress("0x0"), parsedAbi, nil, nil, nil)

	expectedReceivedMap := map[string]interface{}{
		"content": hash,
		"sender":  common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount":  big.NewInt(1),
		"memo":    []byte{88},
	}
	unpackAndCheck(t, bc, expectedReceivedMap, mockLog)
}

func TestTransactGasFee(t *testing.T) {
	assert := assert.New(t)

	// GasTipCap and GasFeeCap
	// When opts.GasTipCap and opts.GasFeeCap are nil
	mt := &mockTransactor{baseFee: big.NewInt(100), gasTipCap: big.NewInt(5)}
	bc := bind.NewBoundContract(common.Address{}, abi.ABI{}, nil, mt, nil)
	opts := &bind.TransactOpts{Signer: mockSign}
	tx, err := bc.Transact(opts, "")
	assert.Nil(err)
	assert.Equal(big.NewInt(5), tx.GasTipCap())
	assert.Equal(big.NewInt(205), tx.GasFeeCap())
	assert.Nil(opts.GasTipCap)
	assert.Nil(opts.GasFeeCap)
	assert.True(mt.suggestGasTipCapCalled)

	// Second call to Transact should use latest suggested GasTipCap
	mt.gasTipCap = big.NewInt(6)
	mt.suggestGasTipCapCalled = false
	tx, err = bc.Transact(opts, "")
	assert.Nil(err)
	assert.Equal(big.NewInt(6), tx.GasTipCap())
	assert.Equal(big.NewInt(206), tx.GasFeeCap())
	assert.True(mt.suggestGasTipCapCalled)

	// GasPrice
	// When opts.GasPrice is nil
	mt = &mockTransactor{gasPrice: big.NewInt(5)}
	bc = bind.NewBoundContract(common.Address{}, abi.ABI{}, nil, mt, nil)
	opts = &bind.TransactOpts{Signer: mockSign}
	tx, err = bc.Transact(opts, "")
	assert.Nil(err)
	assert.Equal(big.NewInt(5), tx.GasPrice())
	assert.Nil(opts.GasPrice)
	assert.True(mt.suggestGasPriceCalled)

	// Second call to Transact should use latest suggested GasPrice
	mt.gasPrice = big.NewInt(6)
	mt.suggestGasPriceCalled = false
	tx, err = bc.Transact(opts, "")
	assert.Nil(err)
	assert.Equal(big.NewInt(6), tx.GasPrice())
	assert.True(mt.suggestGasPriceCalled)
}

func unpackAndCheck(t *testing.T, bc *bind.BoundContract, expected map[string]interface{}, mockLog types.Log) {
	received := make(map[string]interface{})
	if err := bc.UnpackLogIntoMap(received, "received", mockLog); err != nil {
		t.Error(err)
	}

	if len(received) != len(expected) {
		t.Fatalf("unpacked map length %v not equal expected length of %v", len(received), len(expected))
	}
	for name, elem := range expected {
		if !reflect.DeepEqual(elem, received[name]) {
			t.Errorf("field %v does not match expected, want %v, got %v", name, elem, received[name])
		}
	}
}

func newMockLog(topics []common.Hash, txHash common.Hash) types.Log {
	return types.Log{
		Address:     common.HexToAddress("0x0"),
		Topics:      topics,
		Data:        hexutil.MustDecode(hexData),
		BlockNumber: uint64(26),
		TxHash:      txHash,
		TxIndex:     111,
		BlockHash:   common.BytesToHash([]byte{1, 2, 3, 4, 5}),
		Index:       7,
		Removed:     false,
	}
}

func TestCall(t *testing.T) {
	var method, methodWithArg = "something", "somethingArrrrg"
	tests := []struct {
		name, method string
		opts         *bind.CallOpts
		mc           bind.ContractCaller
		results      *[]interface{}
		wantErr      bool
		wantErrExact error
	}{{
		name: "ok not pending",
		mc: &mockCaller{
			codeAtBytes: []byte{0},
		},
		method: method,
	}, {
		name: "ok pending",
		mc: &mockPendingCaller{
			pendingCodeAtBytes: []byte{0},
		},
		opts: &bind.CallOpts{
			Pending: true,
		},
		method: method,
	}, {
		name: "ok hash",
		mc: &mockBlockHashCaller{
			codeAtHashBytes: []byte{0},
		},
		opts: &bind.CallOpts{
			BlockHash: common.Hash{0xaa},
		},
		method: method,
	}, {
		name:    "pack error, no method",
		mc:      new(mockCaller),
		method:  "else",
		wantErr: true,
	}, {
		name: "interface error, pending but not a PendingContractCaller",
		mc:   new(mockCaller),
		opts: &bind.CallOpts{
			Pending: true,
		},
		method:       method,
		wantErrExact: bind.ErrNoPendingState,
	}, {
		name: "interface error, blockHash but not a BlockHashContractCaller",
		mc:   new(mockCaller),
		opts: &bind.CallOpts{
			BlockHash: common.Hash{0xaa},
		},
		method:       method,
		wantErrExact: bind.ErrNoBlockHashState,
	}, {
		name: "pending call canceled",
		mc: &mockPendingCaller{
			pendingCallContractErr: context.DeadlineExceeded,
		},
		opts: &bind.CallOpts{
			Pending: true,
		},
		method:       method,
		wantErrExact: context.DeadlineExceeded,
	}, {
		name: "pending code at error",
		mc: &mockPendingCaller{
			pendingCodeAtErr: errors.New(""),
		},
		opts: &bind.CallOpts{
			Pending: true,
		},
		method:  method,
		wantErr: true,
	}, {
		name: "no pending code at",
		mc:   new(mockPendingCaller),
		opts: &bind.CallOpts{
			Pending: true,
		},
		method:       method,
		wantErrExact: bind.ErrNoCode,
	}, {
		name: "call contract error",
		mc: &mockCaller{
			callContractErr: context.DeadlineExceeded,
		},
		method:       method,
		wantErrExact: context.DeadlineExceeded,
	}, {
		name: "code at error",
		mc: &mockCaller{
			codeAtErr: errors.New(""),
		},
		method:  method,
		wantErr: true,
	}, {
		name:         "no code at",
		mc:           new(mockCaller),
		method:       method,
		wantErrExact: bind.ErrNoCode,
	}, {
		name: "call contract at hash error",
		mc: &mockBlockHashCaller{
			callContractAtHashErr: context.DeadlineExceeded,
		},
		opts: &bind.CallOpts{
			BlockHash: common.Hash{0xaa},
		},
		method:       method,
		wantErrExact: context.DeadlineExceeded,
	}, {
		name: "code at error",
		mc: &mockBlockHashCaller{
			codeAtHashErr: errors.New(""),
		},
		opts: &bind.CallOpts{
			BlockHash: common.Hash{0xaa},
		},
		method:  method,
		wantErr: true,
	}, {
		name: "no code at hash",
		mc:   new(mockBlockHashCaller),
		opts: &bind.CallOpts{
			BlockHash: common.Hash{0xaa},
		},
		method:       method,
		wantErrExact: bind.ErrNoCode,
	}, {
		name: "unpack error missing arg",
		mc: &mockCaller{
			codeAtBytes: []byte{0},
		},
		method:  methodWithArg,
		wantErr: true,
	}, {
		name: "interface unpack error",
		mc: &mockCaller{
			codeAtBytes: []byte{0},
		},
		method:  method,
		results: &[]interface{}{0},
		wantErr: true,
	}}
	for _, test := range tests {
		bc := bind.NewBoundContract(common.HexToAddress("0x0"), abi.ABI{
			Methods: map[string]abi.Method{
				method: {
					Name:    method,
					Outputs: abi.Arguments{},
				},
				methodWithArg: {
					Name:    methodWithArg,
					Outputs: abi.Arguments{abi.Argument{}},
				},
			},
		}, test.mc, nil, nil)
		err := bc.Call(test.opts, test.results, test.method)
		if test.wantErr || test.wantErrExact != nil {
			if err == nil {
				t.Fatalf("%q expected error", test.name)
			}
			if test.wantErrExact != nil && !errors.Is(err, test.wantErrExact) {
				t.Fatalf("%q expected error %q but got %q", test.name, test.wantErrExact, err)
			}
			continue
		}
		if err != nil {
			t.Fatalf("%q unexpected error: %v", test.name, err)
		}
	}
}

// TestCrashers contains some strings which previously caused the abi codec to crash.
func TestCrashers(t *testing.T) {
	abi.JSON(strings.NewReader(`[{"inputs":[{"type":"tuple[]","components":[{"type":"bool","name":"_1"}]}]}]`))
	abi.JSON(strings.NewReader(`[{"inputs":[{"type":"tuple[]","components":[{"type":"bool","name":"&"}]}]}]`))
	abi.JSON(strings.NewReader(`[{"inputs":[{"type":"tuple[]","components":[{"type":"bool","name":"----"}]}]}]`))
	abi.JSON(strings.NewReader(`[{"inputs":[{"type":"tuple[]","components":[{"type":"bool","name":"foo.Bar"}]}]}]`))
}
