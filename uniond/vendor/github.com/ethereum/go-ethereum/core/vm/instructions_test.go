// Copyright 2017 The go-ethereum Authors
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

package vm

import (
	"bytes"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/params"
	"github.com/holiman/uint256"
)

type TwoOperandTestcase struct {
	X        string
	Y        string
	Expected string
}

type twoOperandParams struct {
	x string
	y string
}

var alphabetSoup = "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"
var commonParams []*twoOperandParams
var twoOpMethods map[string]executionFunc

type contractRef struct {
	addr common.Address
}

func (c contractRef) Address() common.Address {
	return c.addr
}

func init() {
	// Params is a list of common edgecases that should be used for some common tests
	params := []string{
		"0000000000000000000000000000000000000000000000000000000000000000", // 0
		"0000000000000000000000000000000000000000000000000000000000000001", // +1
		"0000000000000000000000000000000000000000000000000000000000000005", // +5
		"7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", // + max -1
		"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // + max
		"8000000000000000000000000000000000000000000000000000000000000000", // - max
		"8000000000000000000000000000000000000000000000000000000000000001", // - max+1
		"fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb", // - 5
		"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // - 1
	}
	// Params are combined so each param is used on each 'side'
	commonParams = make([]*twoOperandParams, len(params)*len(params))
	for i, x := range params {
		for j, y := range params {
			commonParams[i*len(params)+j] = &twoOperandParams{x, y}
		}
	}
	twoOpMethods = map[string]executionFunc{
		"add":     opAdd,
		"sub":     opSub,
		"mul":     opMul,
		"div":     opDiv,
		"sdiv":    opSdiv,
		"mod":     opMod,
		"smod":    opSmod,
		"exp":     opExp,
		"signext": opSignExtend,
		"lt":      opLt,
		"gt":      opGt,
		"slt":     opSlt,
		"sgt":     opSgt,
		"eq":      opEq,
		"and":     opAnd,
		"or":      opOr,
		"xor":     opXor,
		"byte":    opByte,
		"shl":     opSHL,
		"shr":     opSHR,
		"sar":     opSAR,
	}
}

func testTwoOperandOp(t *testing.T, tests []TwoOperandTestcase, opFn executionFunc, name string) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		pc             = uint64(0)
		evmInterpreter = env.interpreter
	)

	for i, test := range tests {
		x := new(uint256.Int).SetBytes(common.Hex2Bytes(test.X))
		y := new(uint256.Int).SetBytes(common.Hex2Bytes(test.Y))
		expected := new(uint256.Int).SetBytes(common.Hex2Bytes(test.Expected))
		stack.push(x)
		stack.push(y)
		opFn(&pc, evmInterpreter, &ScopeContext{nil, stack, nil})
		if len(stack.data) != 1 {
			t.Errorf("Expected one item on stack after %v, got %d: ", name, len(stack.data))
		}
		actual := stack.pop()

		if actual.Cmp(expected) != 0 {
			t.Errorf("Testcase %v %d, %v(%x, %x): expected  %x, got %x", name, i, name, x, y, expected, actual)
		}
	}
}

func TestByteOp(t *testing.T) {
	tests := []TwoOperandTestcase{
		{"ABCDEF0908070605040302010000000000000000000000000000000000000000", "00", "AB"},
		{"ABCDEF0908070605040302010000000000000000000000000000000000000000", "01", "CD"},
		{"00CDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff", "00", "00"},
		{"00CDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff", "01", "CD"},
		{"0000000000000000000000000000000000000000000000000000000000102030", "1F", "30"},
		{"0000000000000000000000000000000000000000000000000000000000102030", "1E", "20"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "20", "00"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "FFFFFFFFFFFFFFFF", "00"},
	}
	testTwoOperandOp(t, tests, opByte, "byte")
}

func TestSHL(t *testing.T) {
	// Testcases from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-145.md#shl-shift-left
	tests := []TwoOperandTestcase{
		{"0000000000000000000000000000000000000000000000000000000000000001", "01", "0000000000000000000000000000000000000000000000000000000000000002"},
		{"0000000000000000000000000000000000000000000000000000000000000001", "ff", "8000000000000000000000000000000000000000000000000000000000000000"},
		{"0000000000000000000000000000000000000000000000000000000000000001", "0100", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"0000000000000000000000000000000000000000000000000000000000000001", "0101", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "00", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "01", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "ff", "8000000000000000000000000000000000000000000000000000000000000000"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0100", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"0000000000000000000000000000000000000000000000000000000000000000", "01", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "01", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"},
	}
	testTwoOperandOp(t, tests, opSHL, "shl")
}

func TestSHR(t *testing.T) {
	// Testcases from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-145.md#shr-logical-shift-right
	tests := []TwoOperandTestcase{
		{"0000000000000000000000000000000000000000000000000000000000000001", "00", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"0000000000000000000000000000000000000000000000000000000000000001", "01", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "01", "4000000000000000000000000000000000000000000000000000000000000000"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "ff", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "0100", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "0101", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "00", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "01", "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "ff", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0100", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"0000000000000000000000000000000000000000000000000000000000000000", "01", "0000000000000000000000000000000000000000000000000000000000000000"},
	}
	testTwoOperandOp(t, tests, opSHR, "shr")
}

func TestSAR(t *testing.T) {
	// Testcases from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-145.md#sar-arithmetic-shift-right
	tests := []TwoOperandTestcase{
		{"0000000000000000000000000000000000000000000000000000000000000001", "00", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"0000000000000000000000000000000000000000000000000000000000000001", "01", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "01", "c000000000000000000000000000000000000000000000000000000000000000"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "ff", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "0100", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"8000000000000000000000000000000000000000000000000000000000000000", "0101", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "00", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "01", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "ff", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0100", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0000000000000000000000000000000000000000000000000000000000000000", "01", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"4000000000000000000000000000000000000000000000000000000000000000", "fe", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "f8", "000000000000000000000000000000000000000000000000000000000000007f"},
		{"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "fe", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "ff", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0100", "0000000000000000000000000000000000000000000000000000000000000000"},
	}

	testTwoOperandOp(t, tests, opSAR, "sar")
}

func TestAddMod(t *testing.T) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		evmInterpreter = NewEVMInterpreter(env)
		pc             = uint64(0)
	)
	tests := []struct {
		x        string
		y        string
		z        string
		expected string
	}{
		{"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
			"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
		},
	}
	// x + y = 0x1fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd
	// in 256 bit repr, fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd

	for i, test := range tests {
		x := new(uint256.Int).SetBytes(common.Hex2Bytes(test.x))
		y := new(uint256.Int).SetBytes(common.Hex2Bytes(test.y))
		z := new(uint256.Int).SetBytes(common.Hex2Bytes(test.z))
		expected := new(uint256.Int).SetBytes(common.Hex2Bytes(test.expected))
		stack.push(z)
		stack.push(y)
		stack.push(x)
		opAddmod(&pc, evmInterpreter, &ScopeContext{nil, stack, nil})
		actual := stack.pop()
		if actual.Cmp(expected) != 0 {
			t.Errorf("Testcase %d, expected  %x, got %x", i, expected, actual)
		}
	}
}

// utility function to fill the json-file with testcases
// Enable this test to generate the 'testcases_xx.json' files
func TestWriteExpectedValues(t *testing.T) {
	t.Skip("Enable this test to create json test cases.")

	// getResult is a convenience function to generate the expected values
	getResult := func(args []*twoOperandParams, opFn executionFunc) []TwoOperandTestcase {
		var (
			env         = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
			stack       = newstack()
			pc          = uint64(0)
			interpreter = env.interpreter
		)
		result := make([]TwoOperandTestcase, len(args))
		for i, param := range args {
			x := new(uint256.Int).SetBytes(common.Hex2Bytes(param.x))
			y := new(uint256.Int).SetBytes(common.Hex2Bytes(param.y))
			stack.push(x)
			stack.push(y)
			opFn(&pc, interpreter, &ScopeContext{nil, stack, nil})
			actual := stack.pop()
			result[i] = TwoOperandTestcase{param.x, param.y, fmt.Sprintf("%064x", actual)}
		}
		return result
	}

	for name, method := range twoOpMethods {
		data, err := json.Marshal(getResult(commonParams, method))
		if err != nil {
			t.Fatal(err)
		}
		_ = os.WriteFile(fmt.Sprintf("testdata/testcases_%v.json", name), data, 0644)
		if err != nil {
			t.Fatal(err)
		}
	}
}

// TestJsonTestcases runs through all the testcases defined as json-files
func TestJsonTestcases(t *testing.T) {
	for name := range twoOpMethods {
		data, err := os.ReadFile(fmt.Sprintf("testdata/testcases_%v.json", name))
		if err != nil {
			t.Fatal("Failed to read file", err)
		}
		var testcases []TwoOperandTestcase
		json.Unmarshal(data, &testcases)
		testTwoOperandOp(t, testcases, twoOpMethods[name], name)
	}
}

func opBenchmark(bench *testing.B, op executionFunc, args ...string) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		scope          = &ScopeContext{nil, stack, nil}
		evmInterpreter = NewEVMInterpreter(env)
	)

	env.interpreter = evmInterpreter
	// convert args
	intArgs := make([]*uint256.Int, len(args))
	for i, arg := range args {
		intArgs[i] = new(uint256.Int).SetBytes(common.Hex2Bytes(arg))
	}
	pc := uint64(0)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		for _, arg := range intArgs {
			stack.push(arg)
		}
		op(&pc, evmInterpreter, scope)
		stack.pop()
	}
	bench.StopTimer()

	for i, arg := range args {
		want := new(uint256.Int).SetBytes(common.Hex2Bytes(arg))
		if have := intArgs[i]; !want.Eq(have) {
			bench.Fatalf("input #%d mutated, have %x want %x", i, have, want)
		}
	}
}

func BenchmarkOpAdd64(b *testing.B) {
	x := "ffffffff"
	y := "fd37f3e2bba2c4f"

	opBenchmark(b, opAdd, x, y)
}

func BenchmarkOpAdd128(b *testing.B) {
	x := "ffffffffffffffff"
	y := "f5470b43c6549b016288e9a65629687"

	opBenchmark(b, opAdd, x, y)
}

func BenchmarkOpAdd256(b *testing.B) {
	x := "0802431afcbce1fc194c9eaa417b2fb67dc75a95db0bc7ec6b1c8af11df6a1da9"
	y := "a1f5aac137876480252e5dcac62c354ec0d42b76b0642b6181ed099849ea1d57"

	opBenchmark(b, opAdd, x, y)
}

func BenchmarkOpSub64(b *testing.B) {
	x := "51022b6317003a9d"
	y := "a20456c62e00753a"

	opBenchmark(b, opSub, x, y)
}

func BenchmarkOpSub128(b *testing.B) {
	x := "4dde30faaacdc14d00327aac314e915d"
	y := "9bbc61f5559b829a0064f558629d22ba"

	opBenchmark(b, opSub, x, y)
}

func BenchmarkOpSub256(b *testing.B) {
	x := "4bfcd8bb2ac462735b48a17580690283980aa2d679f091c64364594df113ea37"
	y := "97f9b1765588c4e6b69142eb00d20507301545acf3e1238c86c8b29be227d46e"

	opBenchmark(b, opSub, x, y)
}

func BenchmarkOpMul(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opMul, x, y)
}

func BenchmarkOpDiv256(b *testing.B) {
	x := "ff3f9014f20db29ae04af2c2d265de17"
	y := "fe7fb0d1f59dfe9492ffbf73683fd1e870eec79504c60144cc7f5fc2bad1e611"
	opBenchmark(b, opDiv, x, y)
}

func BenchmarkOpDiv128(b *testing.B) {
	x := "fdedc7f10142ff97"
	y := "fbdfda0e2ce356173d1993d5f70a2b11"
	opBenchmark(b, opDiv, x, y)
}

func BenchmarkOpDiv64(b *testing.B) {
	x := "fcb34eb3"
	y := "f97180878e839129"
	opBenchmark(b, opDiv, x, y)
}

func BenchmarkOpSdiv(b *testing.B) {
	x := "ff3f9014f20db29ae04af2c2d265de17"
	y := "fe7fb0d1f59dfe9492ffbf73683fd1e870eec79504c60144cc7f5fc2bad1e611"

	opBenchmark(b, opSdiv, x, y)
}

func BenchmarkOpMod(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opMod, x, y)
}

func BenchmarkOpSmod(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opSmod, x, y)
}

func BenchmarkOpExp(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opExp, x, y)
}

func BenchmarkOpSignExtend(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opSignExtend, x, y)
}

func BenchmarkOpLt(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opLt, x, y)
}

func BenchmarkOpGt(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opGt, x, y)
}

func BenchmarkOpSlt(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opSlt, x, y)
}

func BenchmarkOpSgt(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opSgt, x, y)
}

func BenchmarkOpEq(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opEq, x, y)
}
func BenchmarkOpEq2(b *testing.B) {
	x := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"
	y := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201fffffffe"
	opBenchmark(b, opEq, x, y)
}
func BenchmarkOpAnd(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opAnd, x, y)
}

func BenchmarkOpOr(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opOr, x, y)
}

func BenchmarkOpXor(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opXor, x, y)
}

func BenchmarkOpByte(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup

	opBenchmark(b, opByte, x, y)
}

func BenchmarkOpAddmod(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup
	z := alphabetSoup

	opBenchmark(b, opAddmod, x, y, z)
}

func BenchmarkOpMulmod(b *testing.B) {
	x := alphabetSoup
	y := alphabetSoup
	z := alphabetSoup

	opBenchmark(b, opMulmod, x, y, z)
}

func BenchmarkOpSHL(b *testing.B) {
	x := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"
	y := "ff"

	opBenchmark(b, opSHL, x, y)
}
func BenchmarkOpSHR(b *testing.B) {
	x := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"
	y := "ff"

	opBenchmark(b, opSHR, x, y)
}
func BenchmarkOpSAR(b *testing.B) {
	x := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"
	y := "ff"

	opBenchmark(b, opSAR, x, y)
}
func BenchmarkOpIsZero(b *testing.B) {
	x := "FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"
	opBenchmark(b, opIszero, x)
}

func TestOpMstore(t *testing.T) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		mem            = NewMemory()
		evmInterpreter = NewEVMInterpreter(env)
	)

	env.interpreter = evmInterpreter
	mem.Resize(64)
	pc := uint64(0)
	v := "abcdef00000000000000abba000000000deaf000000c0de00100000000133700"
	stack.push(new(uint256.Int).SetBytes(common.Hex2Bytes(v)))
	stack.push(new(uint256.Int))
	opMstore(&pc, evmInterpreter, &ScopeContext{mem, stack, nil})
	if got := common.Bytes2Hex(mem.GetCopy(0, 32)); got != v {
		t.Fatalf("Mstore fail, got %v, expected %v", got, v)
	}
	stack.push(new(uint256.Int).SetUint64(0x1))
	stack.push(new(uint256.Int))
	opMstore(&pc, evmInterpreter, &ScopeContext{mem, stack, nil})
	if common.Bytes2Hex(mem.GetCopy(0, 32)) != "0000000000000000000000000000000000000000000000000000000000000001" {
		t.Fatalf("Mstore failed to overwrite previous value")
	}
}

func BenchmarkOpMstore(bench *testing.B) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		mem            = NewMemory()
		evmInterpreter = NewEVMInterpreter(env)
	)

	env.interpreter = evmInterpreter
	mem.Resize(64)
	pc := uint64(0)
	memStart := new(uint256.Int)
	value := new(uint256.Int).SetUint64(0x1337)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		stack.push(value)
		stack.push(memStart)
		opMstore(&pc, evmInterpreter, &ScopeContext{mem, stack, nil})
	}
}

func TestOpTstore(t *testing.T) {
	var (
		statedb, _     = state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewMemoryDatabase()), nil)
		env            = NewEVM(BlockContext{}, TxContext{}, statedb, params.TestChainConfig, Config{})
		stack          = newstack()
		mem            = NewMemory()
		evmInterpreter = NewEVMInterpreter(env)
		caller         = common.Address{}
		to             = common.Address{1}
		contractRef    = contractRef{caller}
		contract       = NewContract(contractRef, AccountRef(to), new(big.Int), 0)
		scopeContext   = ScopeContext{mem, stack, contract}
		value          = common.Hex2Bytes("abcdef00000000000000abba000000000deaf000000c0de00100000000133700")
	)

	// Add a stateObject for the caller and the contract being called
	statedb.CreateAccount(caller)
	statedb.CreateAccount(to)

	env.interpreter = evmInterpreter
	pc := uint64(0)
	// push the value to the stack
	stack.push(new(uint256.Int).SetBytes(value))
	// push the location to the stack
	stack.push(new(uint256.Int))
	opTstore(&pc, evmInterpreter, &scopeContext)
	// there should be no elements on the stack after TSTORE
	if stack.len() != 0 {
		t.Fatal("stack wrong size")
	}
	// push the location to the stack
	stack.push(new(uint256.Int))
	opTload(&pc, evmInterpreter, &scopeContext)
	// there should be one element on the stack after TLOAD
	if stack.len() != 1 {
		t.Fatal("stack wrong size")
	}
	val := stack.peek()
	if !bytes.Equal(val.Bytes(), value) {
		t.Fatal("incorrect element read from transient storage")
	}
}

func BenchmarkOpKeccak256(bench *testing.B) {
	var (
		env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
		stack          = newstack()
		mem            = NewMemory()
		evmInterpreter = NewEVMInterpreter(env)
	)
	env.interpreter = evmInterpreter
	mem.Resize(32)
	pc := uint64(0)
	start := new(uint256.Int)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		stack.push(uint256.NewInt(32))
		stack.push(start)
		opKeccak256(&pc, evmInterpreter, &ScopeContext{mem, stack, nil})
	}
}

func TestCreate2Addreses(t *testing.T) {
	type testcase struct {
		origin   string
		salt     string
		code     string
		expected string
	}

	for i, tt := range []testcase{
		{
			origin:   "0x0000000000000000000000000000000000000000",
			salt:     "0x0000000000000000000000000000000000000000",
			code:     "0x00",
			expected: "0x4d1a2e2bb4f88f0250f26ffff098b0b30b26bf38",
		},
		{
			origin:   "0xdeadbeef00000000000000000000000000000000",
			salt:     "0x0000000000000000000000000000000000000000",
			code:     "0x00",
			expected: "0xB928f69Bb1D91Cd65274e3c79d8986362984fDA3",
		},
		{
			origin:   "0xdeadbeef00000000000000000000000000000000",
			salt:     "0xfeed000000000000000000000000000000000000",
			code:     "0x00",
			expected: "0xD04116cDd17beBE565EB2422F2497E06cC1C9833",
		},
		{
			origin:   "0x0000000000000000000000000000000000000000",
			salt:     "0x0000000000000000000000000000000000000000",
			code:     "0xdeadbeef",
			expected: "0x70f2b2914A2a4b783FaEFb75f459A580616Fcb5e",
		},
		{
			origin:   "0x00000000000000000000000000000000deadbeef",
			salt:     "0xcafebabe",
			code:     "0xdeadbeef",
			expected: "0x60f3f640a8508fC6a86d45DF051962668E1e8AC7",
		},
		{
			origin:   "0x00000000000000000000000000000000deadbeef",
			salt:     "0xcafebabe",
			code:     "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
			expected: "0x1d8bfDC5D46DC4f61D6b6115972536eBE6A8854C",
		},
		{
			origin:   "0x0000000000000000000000000000000000000000",
			salt:     "0x0000000000000000000000000000000000000000",
			code:     "0x",
			expected: "0xE33C0C7F7df4809055C3ebA6c09CFe4BaF1BD9e0",
		},
	} {
		origin := common.BytesToAddress(common.FromHex(tt.origin))
		salt := common.BytesToHash(common.FromHex(tt.salt))
		code := common.FromHex(tt.code)
		codeHash := crypto.Keccak256(code)
		address := crypto.CreateAddress2(origin, salt, codeHash)
		/*
			stack          := newstack()
			// salt, but we don't need that for this test
			stack.push(big.NewInt(int64(len(code)))) //size
			stack.push(big.NewInt(0)) // memstart
			stack.push(big.NewInt(0)) // value
			gas, _ := gasCreate2(params.GasTable{}, nil, nil, stack, nil, 0)
			fmt.Printf("Example %d\n* address `0x%x`\n* salt `0x%x`\n* init_code `0x%x`\n* gas (assuming no mem expansion): `%v`\n* result: `%s`\n\n", i,origin, salt, code, gas, address.String())
		*/
		expected := common.BytesToAddress(common.FromHex(tt.expected))
		if !bytes.Equal(expected.Bytes(), address.Bytes()) {
			t.Errorf("test %d: expected %s, got %s", i, expected.String(), address.String())
		}
	}
}

func TestRandom(t *testing.T) {
	type testcase struct {
		name   string
		random common.Hash
	}

	for _, tt := range []testcase{
		{name: "empty hash", random: common.Hash{}},
		{name: "1", random: common.Hash{0}},
		{name: "emptyCodeHash", random: types.EmptyCodeHash},
		{name: "hash(0x010203)", random: crypto.Keccak256Hash([]byte{0x01, 0x02, 0x03})},
	} {
		var (
			env            = NewEVM(BlockContext{Random: &tt.random}, TxContext{}, nil, params.TestChainConfig, Config{})
			stack          = newstack()
			pc             = uint64(0)
			evmInterpreter = env.interpreter
		)
		opRandom(&pc, evmInterpreter, &ScopeContext{nil, stack, nil})
		if len(stack.data) != 1 {
			t.Errorf("Expected one item on stack after %v, got %d: ", tt.name, len(stack.data))
		}
		actual := stack.pop()
		expected, overflow := uint256.FromBig(new(big.Int).SetBytes(tt.random.Bytes()))
		if overflow {
			t.Errorf("Testcase %v: invalid overflow", tt.name)
		}
		if actual.Cmp(expected) != 0 {
			t.Errorf("Testcase %v: expected  %x, got %x", tt.name, expected, actual)
		}
	}
}

func TestBlobHash(t *testing.T) {
	type testcase struct {
		name   string
		idx    uint64
		expect common.Hash
		hashes []common.Hash
	}
	var (
		zero  = common.Hash{0}
		one   = common.Hash{1}
		two   = common.Hash{2}
		three = common.Hash{3}
	)
	for _, tt := range []testcase{
		{name: "[{1}]", idx: 0, expect: one, hashes: []common.Hash{one}},
		{name: "[1,{2},3]", idx: 2, expect: three, hashes: []common.Hash{one, two, three}},
		{name: "out-of-bounds (empty)", idx: 10, expect: zero, hashes: []common.Hash{}},
		{name: "out-of-bounds", idx: 25, expect: zero, hashes: []common.Hash{one, two, three}},
		{name: "out-of-bounds (nil)", idx: 25, expect: zero, hashes: nil},
	} {
		var (
			env            = NewEVM(BlockContext{}, TxContext{BlobHashes: tt.hashes}, nil, params.TestChainConfig, Config{})
			stack          = newstack()
			pc             = uint64(0)
			evmInterpreter = env.interpreter
		)
		stack.push(uint256.NewInt(tt.idx))
		opBlobHash(&pc, evmInterpreter, &ScopeContext{nil, stack, nil})
		if len(stack.data) != 1 {
			t.Errorf("Expected one item on stack after %v, got %d: ", tt.name, len(stack.data))
		}
		actual := stack.pop()
		expected, overflow := uint256.FromBig(new(big.Int).SetBytes(tt.expect.Bytes()))
		if overflow {
			t.Errorf("Testcase %v: invalid overflow", tt.name)
		}
		if actual.Cmp(expected) != 0 {
			t.Errorf("Testcase %v: expected  %x, got %x", tt.name, expected, actual)
		}
	}
}

func TestOpMCopy(t *testing.T) {
	// Test cases from https://eips.ethereum.org/EIPS/eip-5656#test-cases
	for i, tc := range []struct {
		dst, src, len string
		pre           string
		want          string
		wantGas       uint64
	}{
		{ // MCOPY 0 32 32 - copy 32 bytes from offset 32 to offset 0.
			dst: "0x0", src: "0x20", len: "0x20",
			pre:     "0000000000000000000000000000000000000000000000000000000000000000 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
			want:    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
			wantGas: 6,
		},

		{ // MCOPY 0 0 32 - copy 32 bytes from offset 0 to offset 0.
			dst: "0x0", src: "0x0", len: "0x20",
			pre:     "0101010101010101010101010101010101010101010101010101010101010101",
			want:    "0101010101010101010101010101010101010101010101010101010101010101",
			wantGas: 6,
		},
		{ // MCOPY 0 1 8 - copy 8 bytes from offset 1 to offset 0 (overlapping).
			dst: "0x0", src: "0x1", len: "0x8",
			pre:     "000102030405060708 000000000000000000000000000000000000000000000000",
			want:    "010203040506070808 000000000000000000000000000000000000000000000000",
			wantGas: 6,
		},
		{ // MCOPY 1 0 8 - copy 8 bytes from offset 0 to offset 1 (overlapping).
			dst: "0x1", src: "0x0", len: "0x8",
			pre:     "000102030405060708 000000000000000000000000000000000000000000000000",
			want:    "000001020304050607 000000000000000000000000000000000000000000000000",
			wantGas: 6,
		},
		// Tests below are not in the EIP, but maybe should be added
		{ // MCOPY 0xFFFFFFFFFFFF 0xFFFFFFFFFFFF 0 - copy zero bytes from out-of-bounds index(overlapping).
			dst: "0xFFFFFFFFFFFF", src: "0xFFFFFFFFFFFF", len: "0x0",
			pre:     "11",
			want:    "11",
			wantGas: 3,
		},
		{ // MCOPY 0xFFFFFFFFFFFF 0 0 - copy zero bytes from start of mem to out-of-bounds.
			dst: "0xFFFFFFFFFFFF", src: "0x0", len: "0x0",
			pre:     "11",
			want:    "11",
			wantGas: 3,
		},
		{ // MCOPY 0 0xFFFFFFFFFFFF 0 - copy zero bytes from out-of-bounds to start of mem
			dst: "0x0", src: "0xFFFFFFFFFFFF", len: "0x0",
			pre:     "11",
			want:    "11",
			wantGas: 3,
		},
		{ // MCOPY - copy 1 from space outside of uint64  space
			dst: "0x0", src: "0x10000000000000000", len: "0x1",
			pre: "0",
		},
		{ // MCOPY - copy 1 from 0 to space outside of uint64
			dst: "0x10000000000000000", src: "0x0", len: "0x1",
			pre: "0",
		},
		{ // MCOPY - copy nothing from 0 to space outside of uint64
			dst: "0x10000000000000000", src: "0x0", len: "0x0",
			pre:     "",
			want:    "",
			wantGas: 3,
		},
		{ // MCOPY - copy 1 from 0x20 to 0x10, with no prior allocated mem
			dst: "0x10", src: "0x20", len: "0x1",
			pre: "",
			// 64 bytes
			want:    "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
			wantGas: 12,
		},
		{ // MCOPY - copy 1 from 0x19 to 0x10, with no prior allocated mem
			dst: "0x10", src: "0x19", len: "0x1",
			pre: "",
			// 32 bytes
			want:    "0x0000000000000000000000000000000000000000000000000000000000000000",
			wantGas: 9,
		},
	} {
		var (
			env            = NewEVM(BlockContext{}, TxContext{}, nil, params.TestChainConfig, Config{})
			stack          = newstack()
			pc             = uint64(0)
			evmInterpreter = env.interpreter
		)
		data := common.FromHex(strings.ReplaceAll(tc.pre, " ", ""))
		// Set pre
		mem := NewMemory()
		mem.Resize(uint64(len(data)))
		mem.Set(0, uint64(len(data)), data)
		// Push stack args
		len, _ := uint256.FromHex(tc.len)
		src, _ := uint256.FromHex(tc.src)
		dst, _ := uint256.FromHex(tc.dst)

		stack.push(len)
		stack.push(src)
		stack.push(dst)
		wantErr := (tc.wantGas == 0)
		// Calc mem expansion
		var memorySize uint64
		if memSize, overflow := memoryMcopy(stack); overflow {
			if wantErr {
				continue
			}
			t.Errorf("overflow")
		} else {
			var overflow bool
			if memorySize, overflow = math.SafeMul(toWordSize(memSize), 32); overflow {
				t.Error(ErrGasUintOverflow)
			}
		}
		// and the dynamic cost
		var haveGas uint64
		if dynamicCost, err := gasMcopy(env, nil, stack, mem, memorySize); err != nil {
			t.Error(err)
		} else {
			haveGas = GasFastestStep + dynamicCost
		}
		// Expand mem
		if memorySize > 0 {
			mem.Resize(memorySize)
		}
		// Do the copy
		opMcopy(&pc, evmInterpreter, &ScopeContext{mem, stack, nil})
		want := common.FromHex(strings.ReplaceAll(tc.want, " ", ""))
		if have := mem.store; !bytes.Equal(want, have) {
			t.Errorf("case %d: \nwant: %#x\nhave: %#x\n", i, want, have)
		}
		wantGas := tc.wantGas
		if haveGas != wantGas {
			t.Errorf("case %d: gas wrong, want %d have %d\n", i, wantGas, haveGas)
		}
	}
}
