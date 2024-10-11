// Copyright 2020 The go-ethereum Authors
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

package abi

import (
	"fmt"
	"reflect"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi"
	fuzz "github.com/google/gofuzz"
)

// TestReplicate can be used to replicate crashers from the fuzzing tests.
// Just replace testString with the data in .quoted
func TestReplicate(t *testing.T) {
	testString := "\x20\x20\x20\x20\x20\x20\x20\x20\x80\x00\x00\x00\x20\x20\x20\x20\x00"
	data := []byte(testString)
	fuzzAbi(data)
}

func Fuzz(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzAbi(data)
	})
}

var (
	names            = []string{"_name", "name", "NAME", "name_", "__", "_name_", "n"}
	stateMut         = []string{"", "pure", "view", "payable"}
	stateMutabilites = []*string{&stateMut[0], &stateMut[1], &stateMut[2], &stateMut[3]}
	pays             = []string{"", "true", "false"}
	payables         = []*string{&pays[0], &pays[1]}
	vNames           = []string{"a", "b", "c", "d", "e", "f", "g"}
	varNames         = append(vNames, names...)
	varTypes         = []string{"bool", "address", "bytes", "string",
		"uint8", "int8", "uint8", "int8", "uint16", "int16",
		"uint24", "int24", "uint32", "int32", "uint40", "int40", "uint48", "int48", "uint56", "int56",
		"uint64", "int64", "uint72", "int72", "uint80", "int80", "uint88", "int88", "uint96", "int96",
		"uint104", "int104", "uint112", "int112", "uint120", "int120", "uint128", "int128", "uint136", "int136",
		"uint144", "int144", "uint152", "int152", "uint160", "int160", "uint168", "int168", "uint176", "int176",
		"uint184", "int184", "uint192", "int192", "uint200", "int200", "uint208", "int208", "uint216", "int216",
		"uint224", "int224", "uint232", "int232", "uint240", "int240", "uint248", "int248", "uint256", "int256",
		"bytes1", "bytes2", "bytes3", "bytes4", "bytes5", "bytes6", "bytes7", "bytes8", "bytes9", "bytes10", "bytes11",
		"bytes12", "bytes13", "bytes14", "bytes15", "bytes16", "bytes17", "bytes18", "bytes19", "bytes20", "bytes21",
		"bytes22", "bytes23", "bytes24", "bytes25", "bytes26", "bytes27", "bytes28", "bytes29", "bytes30", "bytes31",
		"bytes32", "bytes"}
)

func unpackPack(abi abi.ABI, method string, input []byte) ([]interface{}, bool) {
	if out, err := abi.Unpack(method, input); err == nil {
		_, err := abi.Pack(method, out...)
		if err != nil {
			// We have some false positives as we can unpack these type successfully, but not pack them
			if err.Error() == "abi: cannot use []uint8 as type [0]int8 as argument" ||
				err.Error() == "abi: cannot use uint8 as type int8 as argument" {
				return out, false
			}
			panic(err)
		}
		return out, true
	}
	return nil, false
}

func packUnpack(abi abi.ABI, method string, input *[]interface{}) bool {
	if packed, err := abi.Pack(method, input); err == nil {
		outptr := reflect.New(reflect.TypeOf(input))
		err := abi.UnpackIntoInterface(outptr.Interface(), method, packed)
		if err != nil {
			panic(err)
		}
		out := outptr.Elem().Interface()
		if !reflect.DeepEqual(input, out) {
			panic(fmt.Sprintf("unpackPack is not equal, \ninput : %x\noutput: %x", input, out))
		}
		return true
	}
	return false
}

type args struct {
	name string
	typ  string
}

func createABI(name string, stateMutability, payable *string, inputs []args) (abi.ABI, error) {
	sig := fmt.Sprintf(`[{ "type" : "function", "name" : "%v" `, name)
	if stateMutability != nil {
		sig += fmt.Sprintf(`, "stateMutability": "%v" `, *stateMutability)
	}
	if payable != nil {
		sig += fmt.Sprintf(`, "payable": %v `, *payable)
	}
	if len(inputs) > 0 {
		sig += `, "inputs" : [ {`
		for i, inp := range inputs {
			sig += fmt.Sprintf(`"name" : "%v", "type" : "%v" `, inp.name, inp.typ)
			if i+1 < len(inputs) {
				sig += ","
			}
		}
		sig += "} ]"
		sig += `, "outputs" : [ {`
		for i, inp := range inputs {
			sig += fmt.Sprintf(`"name" : "%v", "type" : "%v" `, inp.name, inp.typ)
			if i+1 < len(inputs) {
				sig += ","
			}
		}
		sig += "} ]"
	}
	sig += `}]`

	return abi.JSON(strings.NewReader(sig))
}

func fuzzAbi(input []byte) int {
	good := false
	fuzzer := fuzz.NewFromGoFuzz(input)

	name := names[getUInt(fuzzer)%len(names)]
	stateM := stateMutabilites[getUInt(fuzzer)%len(stateMutabilites)]
	payable := payables[getUInt(fuzzer)%len(payables)]
	maxLen := 5
	for k := 1; k < maxLen; k++ {
		var arg []args
		for i := k; i > 0; i-- {
			argName := varNames[i]
			argTyp := varTypes[getUInt(fuzzer)%len(varTypes)]
			if getUInt(fuzzer)%10 == 0 {
				argTyp += "[]"
			} else if getUInt(fuzzer)%10 == 0 {
				arrayArgs := getUInt(fuzzer)%30 + 1
				argTyp += fmt.Sprintf("[%d]", arrayArgs)
			}
			arg = append(arg, args{
				name: argName,
				typ:  argTyp,
			})
		}
		abi, err := createABI(name, stateM, payable, arg)
		if err != nil {
			continue
		}
		structs, b := unpackPack(abi, name, input)
		c := packUnpack(abi, name, &structs)
		good = good || b || c
	}
	if good {
		return 1
	}
	return 0
}

func getUInt(fuzzer *fuzz.Fuzzer) int {
	var i int
	fuzzer.Fuzz(&i)
	if i < 0 {
		i = -i
		if i < 0 {
			return 0
		}
	}
	return i
}
