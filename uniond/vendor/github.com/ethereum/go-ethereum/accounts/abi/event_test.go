// Copyright 2016 The go-ethereum Authors
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
	"bytes"
	"encoding/hex"
	"encoding/json"
	"math/big"
	"reflect"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var jsonEventTransfer = []byte(`{
  "anonymous": false,
  "inputs": [
    {
      "indexed": true, "name": "from", "type": "address"
    }, {
      "indexed": true, "name": "to", "type": "address"
    }, {
      "indexed": false, "name": "value", "type": "uint256"
  }],
  "name": "Transfer",
  "type": "event"
}`)

var jsonEventPledge = []byte(`{
  "anonymous": false,
  "inputs": [{
      "indexed": false, "name": "who", "type": "address"
    }, {
      "indexed": false, "name": "wad", "type": "uint128"
    }, {
      "indexed": false, "name": "currency", "type": "bytes3"
  }],
  "name": "Pledge",
  "type": "event"
}`)

var jsonEventMixedCase = []byte(`{
	"anonymous": false,
	"inputs": [{
		"indexed": false, "name": "value", "type": "uint256"
	  }, {
		"indexed": false, "name": "_value", "type": "uint256"
	  }, {
		"indexed": false, "name": "Value", "type": "uint256"
	}],
	"name": "MixedCase",
	"type": "event"
  }`)

// 1000000
var transferData1 = "00000000000000000000000000000000000000000000000000000000000f4240"

// "0x00Ce0d46d924CC8437c806721496599FC3FFA268", 2218516807680, "usd"
var pledgeData1 = "00000000000000000000000000ce0d46d924cc8437c806721496599fc3ffa2680000000000000000000000000000000000000000000000000000020489e800007573640000000000000000000000000000000000000000000000000000000000"

// 1000000,2218516807680,1000001
var mixedCaseData1 = "00000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000020489e8000000000000000000000000000000000000000000000000000000000000000f4241"

func TestEventId(t *testing.T) {
	var table = []struct {
		definition   string
		expectations map[string]common.Hash
	}{
		{
			definition: `[
			{ "type" : "event", "name" : "Balance", "inputs": [{ "name" : "in", "type": "uint256" }] },
			{ "type" : "event", "name" : "Check", "inputs": [{ "name" : "t", "type": "address" }, { "name": "b", "type": "uint256" }] }
			]`,
			expectations: map[string]common.Hash{
				"Balance": crypto.Keccak256Hash([]byte("Balance(uint256)")),
				"Check":   crypto.Keccak256Hash([]byte("Check(address,uint256)")),
			},
		},
	}

	for _, test := range table {
		abi, err := JSON(strings.NewReader(test.definition))
		if err != nil {
			t.Fatal(err)
		}

		for name, event := range abi.Events {
			if event.ID != test.expectations[name] {
				t.Errorf("expected id to be %x, got %x", test.expectations[name], event.ID)
			}
		}
	}
}

func TestEventString(t *testing.T) {
	var table = []struct {
		definition   string
		expectations map[string]string
	}{
		{
			definition: `[
			{ "type" : "event", "name" : "Balance", "inputs": [{ "name" : "in", "type": "uint256" }] },
			{ "type" : "event", "name" : "Check", "inputs": [{ "name" : "t", "type": "address" }, { "name": "b", "type": "uint256" }] },
			{ "type" : "event", "name" : "Transfer", "inputs": [{ "name": "from", "type": "address", "indexed": true }, { "name": "to", "type": "address", "indexed": true }, { "name": "value", "type": "uint256" }] }
			]`,
			expectations: map[string]string{
				"Balance":  "event Balance(uint256 in)",
				"Check":    "event Check(address t, uint256 b)",
				"Transfer": "event Transfer(address indexed from, address indexed to, uint256 value)",
			},
		},
	}

	for _, test := range table {
		abi, err := JSON(strings.NewReader(test.definition))
		if err != nil {
			t.Fatal(err)
		}

		for name, event := range abi.Events {
			if event.String() != test.expectations[name] {
				t.Errorf("expected string to be %s, got %s", test.expectations[name], event.String())
			}
		}
	}
}

// TestEventMultiValueWithArrayUnpack verifies that array fields will be counted after parsing array.
func TestEventMultiValueWithArrayUnpack(t *testing.T) {
	definition := `[{"name": "test", "type": "event", "inputs": [{"indexed": false, "name":"value1", "type":"uint8[2]"},{"indexed": false, "name":"value2", "type":"uint8"}]}]`
	abi, err := JSON(strings.NewReader(definition))
	require.NoError(t, err)
	var b bytes.Buffer
	var i uint8 = 1
	for ; i <= 3; i++ {
		b.Write(packNum(reflect.ValueOf(i)))
	}
	unpacked, err := abi.Unpack("test", b.Bytes())
	require.NoError(t, err)
	require.Equal(t, [2]uint8{1, 2}, unpacked[0])
	require.Equal(t, uint8(3), unpacked[1])
}

func TestEventTupleUnpack(t *testing.T) {
	type EventTransfer struct {
		Value *big.Int
	}

	type EventTransferWithTag struct {
		// this is valid because `value` is not exportable,
		// so value is only unmarshalled into `Value1`.
		value  *big.Int //lint:ignore U1000 unused field is part of test
		Value1 *big.Int `abi:"value"`
	}

	type BadEventTransferWithSameFieldAndTag struct {
		Value  *big.Int
		Value1 *big.Int `abi:"value"`
	}

	type BadEventTransferWithDuplicatedTag struct {
		Value1 *big.Int `abi:"value"`
		Value2 *big.Int `abi:"value"`
	}

	type BadEventTransferWithEmptyTag struct {
		Value *big.Int `abi:""`
	}

	type EventPledge struct {
		Who      common.Address
		Wad      *big.Int
		Currency [3]byte
	}

	type BadEventPledge struct {
		Who      string
		Wad      int
		Currency [3]byte
	}

	type EventMixedCase struct {
		Value1 *big.Int `abi:"value"`
		Value2 *big.Int `abi:"_value"`
		Value3 *big.Int `abi:"Value"`
	}

	bigint := new(big.Int)
	bigintExpected := big.NewInt(1000000)
	bigintExpected2 := big.NewInt(2218516807680)
	bigintExpected3 := big.NewInt(1000001)
	addr := common.HexToAddress("0x00Ce0d46d924CC8437c806721496599FC3FFA268")
	var testCases = []struct {
		data     string
		dest     interface{}
		expected interface{}
		jsonLog  []byte
		error    string
		name     string
	}{{
		transferData1,
		&EventTransfer{},
		&EventTransfer{Value: bigintExpected},
		jsonEventTransfer,
		"",
		"Can unpack ERC20 Transfer event into structure",
	}, {
		transferData1,
		&[]interface{}{&bigint},
		&[]interface{}{&bigintExpected},
		jsonEventTransfer,
		"",
		"Can unpack ERC20 Transfer event into slice",
	}, {
		transferData1,
		&EventTransferWithTag{},
		&EventTransferWithTag{Value1: bigintExpected},
		jsonEventTransfer,
		"",
		"Can unpack ERC20 Transfer event into structure with abi: tag",
	}, {
		transferData1,
		&BadEventTransferWithDuplicatedTag{},
		&BadEventTransferWithDuplicatedTag{},
		jsonEventTransfer,
		"struct: abi tag in 'Value2' already mapped",
		"Can not unpack ERC20 Transfer event with duplicated abi tag",
	}, {
		transferData1,
		&BadEventTransferWithSameFieldAndTag{},
		&BadEventTransferWithSameFieldAndTag{},
		jsonEventTransfer,
		"abi: multiple variables maps to the same abi field 'value'",
		"Can not unpack ERC20 Transfer event with a field and a tag mapping to the same abi variable",
	}, {
		transferData1,
		&BadEventTransferWithEmptyTag{},
		&BadEventTransferWithEmptyTag{},
		jsonEventTransfer,
		"struct: abi tag in 'Value' is empty",
		"Can not unpack ERC20 Transfer event with an empty tag",
	}, {
		pledgeData1,
		&EventPledge{},
		&EventPledge{
			addr,
			bigintExpected2,
			[3]byte{'u', 's', 'd'}},
		jsonEventPledge,
		"",
		"Can unpack Pledge event into structure",
	}, {
		pledgeData1,
		&[]interface{}{&common.Address{}, &bigint, &[3]byte{}},
		&[]interface{}{
			&addr,
			&bigintExpected2,
			&[3]byte{'u', 's', 'd'}},
		jsonEventPledge,
		"",
		"Can unpack Pledge event into slice",
	}, {
		pledgeData1,
		&[3]interface{}{&common.Address{}, &bigint, &[3]byte{}},
		&[3]interface{}{
			&addr,
			&bigintExpected2,
			&[3]byte{'u', 's', 'd'}},
		jsonEventPledge,
		"",
		"Can unpack Pledge event into an array",
	}, {
		pledgeData1,
		&[]interface{}{new(int), 0, 0},
		&[]interface{}{},
		jsonEventPledge,
		"abi: cannot unmarshal common.Address in to int",
		"Can not unpack Pledge event into slice with wrong types",
	}, {
		pledgeData1,
		&BadEventPledge{},
		&BadEventPledge{},
		jsonEventPledge,
		"abi: cannot unmarshal common.Address in to string",
		"Can not unpack Pledge event into struct with wrong filed types",
	}, {
		pledgeData1,
		&[]interface{}{common.Address{}, new(big.Int)},
		&[]interface{}{},
		jsonEventPledge,
		"abi: insufficient number of arguments for unpack, want 3, got 2",
		"Can not unpack Pledge event into too short slice",
	}, {
		pledgeData1,
		new(map[string]interface{}),
		&[]interface{}{},
		jsonEventPledge,
		"abi:[2] cannot unmarshal tuple in to map[string]interface {}",
		"Can not unpack Pledge event into map",
	}, {
		mixedCaseData1,
		&EventMixedCase{},
		&EventMixedCase{Value1: bigintExpected, Value2: bigintExpected2, Value3: bigintExpected3},
		jsonEventMixedCase,
		"",
		"Can unpack abi variables with mixed case",
	}}

	for _, tc := range testCases {
		assert := assert.New(t)
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			err := unpackTestEventData(tc.dest, tc.data, tc.jsonLog, assert)
			if tc.error == "" {
				assert.Nil(err, "Should be able to unpack event data.")
				assert.Equal(tc.expected, tc.dest, tc.name)
			} else {
				assert.EqualError(err, tc.error, tc.name)
			}
		})
	}
}

func unpackTestEventData(dest interface{}, hexData string, jsonEvent []byte, assert *assert.Assertions) error {
	data, err := hex.DecodeString(hexData)
	assert.NoError(err, "Hex data should be a correct hex-string")
	var e Event
	assert.NoError(json.Unmarshal(jsonEvent, &e), "Should be able to unmarshal event ABI")
	a := ABI{Events: map[string]Event{"e": e}}
	return a.UnpackIntoInterface(dest, "e", data)
}

// TestEventUnpackIndexed verifies that indexed field will be skipped by event decoder.
func TestEventUnpackIndexed(t *testing.T) {
	definition := `[{"name": "test", "type": "event", "inputs": [{"indexed": true, "name":"value1", "type":"uint8"},{"indexed": false, "name":"value2", "type":"uint8"}]}]`
	type testStruct struct {
		Value1 uint8 // indexed
		Value2 uint8
	}
	abi, err := JSON(strings.NewReader(definition))
	require.NoError(t, err)
	var b bytes.Buffer
	b.Write(packNum(reflect.ValueOf(uint8(8))))
	var rst testStruct
	require.NoError(t, abi.UnpackIntoInterface(&rst, "test", b.Bytes()))
	require.Equal(t, uint8(0), rst.Value1)
	require.Equal(t, uint8(8), rst.Value2)
}

// TestEventIndexedWithArrayUnpack verifies that decoder will not overflow when static array is indexed input.
func TestEventIndexedWithArrayUnpack(t *testing.T) {
	definition := `[{"name": "test", "type": "event", "inputs": [{"indexed": true, "name":"value1", "type":"uint8[2]"},{"indexed": false, "name":"value2", "type":"string"}]}]`
	type testStruct struct {
		Value1 [2]uint8 // indexed
		Value2 string
	}
	abi, err := JSON(strings.NewReader(definition))
	require.NoError(t, err)
	var b bytes.Buffer
	stringOut := "abc"
	// number of fields that will be encoded * 32
	b.Write(packNum(reflect.ValueOf(32)))
	b.Write(packNum(reflect.ValueOf(len(stringOut))))
	b.Write(common.RightPadBytes([]byte(stringOut), 32))

	var rst testStruct
	require.NoError(t, abi.UnpackIntoInterface(&rst, "test", b.Bytes()))
	require.Equal(t, [2]uint8{0, 0}, rst.Value1)
	require.Equal(t, stringOut, rst.Value2)
}
