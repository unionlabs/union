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

package core_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/signer/core"
	"github.com/ethereum/go-ethereum/signer/core/apitypes"
)

var typesStandard = apitypes.Types{
	"EIP712Domain": {
		{
			Name: "name",
			Type: "string",
		},
		{
			Name: "version",
			Type: "string",
		},
		{
			Name: "chainId",
			Type: "uint256",
		},
		{
			Name: "verifyingContract",
			Type: "address",
		},
	},
	"Person": {
		{
			Name: "name",
			Type: "string",
		},
		{
			Name: "wallet",
			Type: "address",
		},
	},
	"Mail": {
		{
			Name: "from",
			Type: "Person",
		},
		{
			Name: "to",
			Type: "Person",
		},
		{
			Name: "contents",
			Type: "string",
		},
	},
}

var jsonTypedData = `
    {
      "types": {
        "EIP712Domain": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "version",
            "type": "string"
          },
          {
            "name": "chainId",
            "type": "uint256"
          },
          {
            "name": "verifyingContract",
            "type": "address"
          }
        ],
        "Person": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "test",
            "type": "uint8"
          },
          {
            "name": "wallet",
            "type": "address"
          }
        ],
        "Mail": [
          {
            "name": "from",
            "type": "Person"
          },
          {
            "name": "to",
            "type": "Person"
          },
          {
            "name": "contents",
            "type": "string"
          }
        ]
      },
      "primaryType": "Mail",
      "domain": {
        "name": "Ether Mail",
        "version": "1",
        "chainId": "1",
        "verifyingContract": "0xCCCcccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
      },
      "message": {
        "from": {
          "name": "Cow",
		  "test": 3,
          "wallet": "0xcD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
        },
        "to": {
          "name": "Bob",
          "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
        },
        "contents": "Hello, Bob!"
      }
    }
`

const primaryType = "Mail"

var domainStandard = apitypes.TypedDataDomain{
	Name:              "Ether Mail",
	Version:           "1",
	ChainId:           math.NewHexOrDecimal256(1),
	VerifyingContract: "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC",
	Salt:              "",
}

var messageStandard = map[string]interface{}{
	"from": map[string]interface{}{
		"name":   "Cow",
		"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
	},
	"to": map[string]interface{}{
		"name":   "Bob",
		"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB",
	},
	"contents": "Hello, Bob!",
}

var typedData = apitypes.TypedData{
	Types:       typesStandard,
	PrimaryType: primaryType,
	Domain:      domainStandard,
	Message:     messageStandard,
}

func TestSignData(t *testing.T) {
	api, control := setup(t)
	//Create two accounts
	createAccount(control, api, t)
	createAccount(control, api, t)
	control.approveCh <- "1"
	list, err := api.List(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	a := common.NewMixedcaseAddress(list[0])

	control.approveCh <- "Y"
	control.inputCh <- "wrongpassword"
	signature, err := api.SignData(context.Background(), apitypes.TextPlain.Mime, a, hexutil.Encode([]byte("EHLO world")))
	if signature != nil {
		t.Errorf("Expected nil-data, got %x", signature)
	}
	if err != keystore.ErrDecrypt {
		t.Errorf("Expected ErrLocked! '%v'", err)
	}
	control.approveCh <- "No way"
	signature, err = api.SignData(context.Background(), apitypes.TextPlain.Mime, a, hexutil.Encode([]byte("EHLO world")))
	if signature != nil {
		t.Errorf("Expected nil-data, got %x", signature)
	}
	if err != core.ErrRequestDenied {
		t.Errorf("Expected ErrRequestDenied! '%v'", err)
	}
	// text/plain
	control.approveCh <- "Y"
	control.inputCh <- "a_long_password"
	signature, err = api.SignData(context.Background(), apitypes.TextPlain.Mime, a, hexutil.Encode([]byte("EHLO world")))
	if err != nil {
		t.Fatal(err)
	}
	if signature == nil || len(signature) != 65 {
		t.Errorf("Expected 65 byte signature (got %d bytes)", len(signature))
	}
	// data/typed via SignTypeData
	control.approveCh <- "Y"
	control.inputCh <- "a_long_password"
	var want []byte
	if signature, err = api.SignTypedData(context.Background(), a, typedData); err != nil {
		t.Fatal(err)
	} else if signature == nil || len(signature) != 65 {
		t.Errorf("Expected 65 byte signature (got %d bytes)", len(signature))
	} else {
		want = signature
	}

	// data/typed via SignData / mimetype typed data
	control.approveCh <- "Y"
	control.inputCh <- "a_long_password"
	if typedDataJson, err := json.Marshal(typedData); err != nil {
		t.Fatal(err)
	} else if signature, err = api.SignData(context.Background(), apitypes.DataTyped.Mime, a, hexutil.Encode(typedDataJson)); err != nil {
		t.Fatal(err)
	} else if signature == nil || len(signature) != 65 {
		t.Errorf("Expected 65 byte signature (got %d bytes)", len(signature))
	} else if have := signature; !bytes.Equal(have, want) {
		t.Fatalf("want %x, have %x", want, have)
	}
}

func TestDomainChainId(t *testing.T) {
	withoutChainID := apitypes.TypedData{
		Types: apitypes.Types{
			"EIP712Domain": []apitypes.Type{
				{Name: "name", Type: "string"},
			},
		},
		Domain: apitypes.TypedDataDomain{
			Name: "test",
		},
	}

	if _, ok := withoutChainID.Domain.Map()["chainId"]; ok {
		t.Errorf("Expected the chainId key to not be present in the domain map")
	}
	// should encode successfully
	if _, err := withoutChainID.HashStruct("EIP712Domain", withoutChainID.Domain.Map()); err != nil {
		t.Errorf("Expected the typedData to encode the domain successfully, got %v", err)
	}
	withChainID := apitypes.TypedData{
		Types: apitypes.Types{
			"EIP712Domain": []apitypes.Type{
				{Name: "name", Type: "string"},
				{Name: "chainId", Type: "uint256"},
			},
		},
		Domain: apitypes.TypedDataDomain{
			Name:    "test",
			ChainId: math.NewHexOrDecimal256(1),
		},
	}

	if _, ok := withChainID.Domain.Map()["chainId"]; !ok {
		t.Errorf("Expected the chainId key be present in the domain map")
	}
	// should encode successfully
	if _, err := withChainID.HashStruct("EIP712Domain", withChainID.Domain.Map()); err != nil {
		t.Errorf("Expected the typedData to encode the domain successfully, got %v", err)
	}
}

func TestHashStruct(t *testing.T) {
	hash, err := typedData.HashStruct(typedData.PrimaryType, typedData.Message)
	if err != nil {
		t.Fatal(err)
	}
	mainHash := fmt.Sprintf("0x%s", common.Bytes2Hex(hash))
	if mainHash != "0xc52c0ee5d84264471806290a3f2c4cecfc5490626bf912d01f240d7a274b371e" {
		t.Errorf("Expected different hashStruct result (got %s)", mainHash)
	}

	hash, err = typedData.HashStruct("EIP712Domain", typedData.Domain.Map())
	if err != nil {
		t.Error(err)
	}
	domainHash := fmt.Sprintf("0x%s", common.Bytes2Hex(hash))
	if domainHash != "0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f" {
		t.Errorf("Expected different domain hashStruct result (got %s)", domainHash)
	}
}

func TestEncodeType(t *testing.T) {
	domainTypeEncoding := string(typedData.EncodeType("EIP712Domain"))
	if domainTypeEncoding != "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)" {
		t.Errorf("Expected different encodeType result (got %s)", domainTypeEncoding)
	}

	mailTypeEncoding := string(typedData.EncodeType(typedData.PrimaryType))
	if mailTypeEncoding != "Mail(Person from,Person to,string contents)Person(string name,address wallet)" {
		t.Errorf("Expected different encodeType result (got %s)", mailTypeEncoding)
	}
}

func TestTypeHash(t *testing.T) {
	mailTypeHash := fmt.Sprintf("0x%s", common.Bytes2Hex(typedData.TypeHash(typedData.PrimaryType)))
	if mailTypeHash != "0xa0cedeb2dc280ba39b857546d74f5549c3a1d7bdc2dd96bf881f76108e23dac2" {
		t.Errorf("Expected different typeHash result (got %s)", mailTypeHash)
	}
}

func TestEncodeData(t *testing.T) {
	hash, err := typedData.EncodeData(typedData.PrimaryType, typedData.Message, 0)
	if err != nil {
		t.Fatal(err)
	}
	dataEncoding := fmt.Sprintf("0x%s", common.Bytes2Hex(hash))
	if dataEncoding != "0xa0cedeb2dc280ba39b857546d74f5549c3a1d7bdc2dd96bf881f76108e23dac2fc71e5fa27ff56c350aa531bc129ebdf613b772b6604664f5d8dbe21b85eb0c8cd54f074a4af31b4411ff6a60c9719dbd559c221c8ac3492d9d872b041d703d1b5aadf3154a261abdd9086fc627b61efca26ae5702701d05cd2305f7c52a2fc8" {
		t.Errorf("Expected different encodeData result (got %s)", dataEncoding)
	}
}

func TestFormatter(t *testing.T) {
	var d apitypes.TypedData
	err := json.Unmarshal([]byte(jsonTypedData), &d)
	if err != nil {
		t.Fatalf("unmarshalling failed '%v'", err)
	}
	formatted, _ := d.Format()
	for _, item := range formatted {
		t.Logf("'%v'\n", item.Pprint(0))
	}

	j, _ := json.Marshal(formatted)
	t.Logf("'%v'\n", string(j))
}

func sign(typedData apitypes.TypedData) ([]byte, []byte, error) {
	domainSeparator, err := typedData.HashStruct("EIP712Domain", typedData.Domain.Map())
	if err != nil {
		return nil, nil, err
	}
	typedDataHash, err := typedData.HashStruct(typedData.PrimaryType, typedData.Message)
	if err != nil {
		return nil, nil, err
	}
	rawData := []byte(fmt.Sprintf("\x19\x01%s%s", string(domainSeparator), string(typedDataHash)))
	sighash := crypto.Keccak256(rawData)
	return typedDataHash, sighash, nil
}

func TestJsonFiles(t *testing.T) {
	testfiles, err := os.ReadDir("testdata/")
	if err != nil {
		t.Fatalf("failed reading files: %v", err)
	}
	for i, fInfo := range testfiles {
		if !strings.HasSuffix(fInfo.Name(), "json") {
			continue
		}
		expectedFailure := strings.HasPrefix(fInfo.Name(), "expfail")
		data, err := os.ReadFile(path.Join("testdata", fInfo.Name()))
		if err != nil {
			t.Errorf("Failed to read file %v: %v", fInfo.Name(), err)
			continue
		}
		var typedData apitypes.TypedData
		err = json.Unmarshal(data, &typedData)
		if err != nil {
			t.Errorf("Test %d, file %v, json unmarshalling failed: %v", i, fInfo.Name(), err)
			continue
		}
		_, _, err = sign(typedData)
		t.Logf("Error %v\n", err)
		if err != nil && !expectedFailure {
			t.Errorf("Test %d failed, file %v: %v", i, fInfo.Name(), err)
		}
		if expectedFailure && err == nil {
			t.Errorf("Test %d succeeded (expected failure), file %v: %v", i, fInfo.Name(), err)
		}
	}
}

// TestFuzzerFiles tests some files that have been found by fuzzing to cause
// crashes or hangs.
func TestFuzzerFiles(t *testing.T) {
	corpusdir := path.Join("testdata", "fuzzing")
	testfiles, err := os.ReadDir(corpusdir)
	if err != nil {
		t.Fatalf("failed reading files: %v", err)
	}
	verbose := false
	for i, fInfo := range testfiles {
		data, err := os.ReadFile(path.Join(corpusdir, fInfo.Name()))
		if err != nil {
			t.Errorf("Failed to read file %v: %v", fInfo.Name(), err)
			continue
		}
		var typedData apitypes.TypedData
		err = json.Unmarshal(data, &typedData)
		if err != nil {
			t.Errorf("Test %d, file %v, json unmarshalling failed: %v", i, fInfo.Name(), err)
			continue
		}
		_, err = typedData.EncodeData("EIP712Domain", typedData.Domain.Map(), 1)
		if verbose && err != nil {
			t.Logf("%d, EncodeData[1] err: %v\n", i, err)
		}
		_, err = typedData.EncodeData(typedData.PrimaryType, typedData.Message, 1)
		if verbose && err != nil {
			t.Logf("%d, EncodeData[2] err: %v\n", i, err)
		}
		typedData.Format()
	}
}

var gnosisTypedData = `
{
	"types": {
		"EIP712Domain": [
			{ "type": "address", "name": "verifyingContract" }
		],
		"SafeTx": [
			{ "type": "address", "name": "to" },
			{ "type": "uint256", "name": "value" },
			{ "type": "bytes", "name": "data" },
			{ "type": "uint8", "name": "operation" },
			{ "type": "uint256", "name": "safeTxGas" },
			{ "type": "uint256", "name": "baseGas" },
			{ "type": "uint256", "name": "gasPrice" },
			{ "type": "address", "name": "gasToken" },
			{ "type": "address", "name": "refundReceiver" },
			{ "type": "uint256", "name": "nonce" }
		]
	},
	"domain": {
		"verifyingContract": "0x25a6c4BBd32B2424A9c99aEB0584Ad12045382B3"
	},
	"primaryType": "SafeTx",
	"message": {
		"to": "0x9eE457023bB3De16D51A003a247BaEaD7fce313D",
		"value": "20000000000000000",
		"data": "0x",
		"operation": 0,
		"safeTxGas": 27845,
		"baseGas": 0,
		"gasPrice": "0",
		"gasToken": "0x0000000000000000000000000000000000000000",
		"refundReceiver": "0x0000000000000000000000000000000000000000",
		"nonce": 3
	}
}`

var gnosisTx = `
{
      "safe": "0x25a6c4BBd32B2424A9c99aEB0584Ad12045382B3",
      "to": "0x9eE457023bB3De16D51A003a247BaEaD7fce313D",
      "value": "20000000000000000",
      "data": null,
      "operation": 0,
      "gasToken": "0x0000000000000000000000000000000000000000",
      "safeTxGas": 27845,
      "baseGas": 0,
      "gasPrice": "0",
      "refundReceiver": "0x0000000000000000000000000000000000000000",
      "nonce": 3,
      "executionDate": null,
      "submissionDate": "2020-09-15T21:59:23.815748Z",
      "modified": "2020-09-15T21:59:23.815748Z",
      "blockNumber": null,
      "transactionHash": null,
      "safeTxHash": "0x28bae2bd58d894a1d9b69e5e9fde3570c4b98a6fc5499aefb54fb830137e831f",
      "executor": null,
      "isExecuted": false,
      "isSuccessful": null,
      "ethGasPrice": null,
      "gasUsed": null,
      "fee": null,
      "origin": null,
      "dataDecoded": null,
      "confirmationsRequired": null,
      "confirmations": [
        {
          "owner": "0xAd2e180019FCa9e55CADe76E4487F126Fd08DA34",
          "submissionDate": "2020-09-15T21:59:28.281243Z",
          "transactionHash": null,
          "confirmationType": "CONFIRMATION",
          "signature": "0x5e562065a0cb15d766dac0cd49eb6d196a41183af302c4ecad45f1a81958d7797753f04424a9b0aa1cb0448e4ec8e189540fbcdda7530ef9b9d95dfc2d36cb521b",
          "signatureType": "EOA"
        }
      ],
      "signatures": null
    }
`

// TestGnosisTypedData tests the scenario where a user submits a full EIP-712
// struct without using the gnosis-specific endpoint
func TestGnosisTypedData(t *testing.T) {
	var td apitypes.TypedData
	err := json.Unmarshal([]byte(gnosisTypedData), &td)
	if err != nil {
		t.Fatalf("unmarshalling failed '%v'", err)
	}
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x28bae2bd58d894a1d9b69e5e9fde3570c4b98a6fc5499aefb54fb830137e831f")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}

// TestGnosisCustomData tests the scenario where a user submits only the gnosis-safe
// specific data, and we fill the TypedData struct on our side
func TestGnosisCustomData(t *testing.T) {
	var tx core.GnosisSafeTx
	err := json.Unmarshal([]byte(gnosisTx), &tx)
	if err != nil {
		t.Fatal(err)
	}
	var td = tx.ToTypedData()
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x28bae2bd58d894a1d9b69e5e9fde3570c4b98a6fc5499aefb54fb830137e831f")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}

var gnosisTypedDataWithChainId = `
{
	"types": {
    "EIP712Domain": [
        { "type": "uint256", "name": "chainId" },
        { "type": "address", "name": "verifyingContract" }
    ],
		"SafeTx": [
			{ "type": "address", "name": "to" },
			{ "type": "uint256", "name": "value" },
			{ "type": "bytes", "name": "data" },
			{ "type": "uint8", "name": "operation" },
			{ "type": "uint256", "name": "safeTxGas" },
			{ "type": "uint256", "name": "baseGas" },
			{ "type": "uint256", "name": "gasPrice" },
			{ "type": "address", "name": "gasToken" },
			{ "type": "address", "name": "refundReceiver" },
			{ "type": "uint256", "name": "nonce" }
		]
	},
	"domain": {
		"verifyingContract": "0x111dAE35D176A9607053e0c46e91F36AFbC1dc57",
		"chainId": "4"
	},
	"primaryType": "SafeTx",
	"message": {
		"to": "0x5592EC0cfb4dbc12D3aB100b257153436a1f0FEa",
		"value": "0",
		"data": "0xa9059cbb00000000000000000000000099d580d3a7fe7bd183b2464517b2cd7ce5a8f15a0000000000000000000000000000000000000000000000000de0b6b3a7640000",
		"operation": 0,
		"safeTxGas": 0,
		"baseGas": 0,
		"gasPrice": "0",
		"gasToken": "0x0000000000000000000000000000000000000000",
		"refundReceiver": "0x0000000000000000000000000000000000000000",
		"nonce": 15
	}
}`

var gnosisTxWithChainId = `
{
	"safe": "0x111dAE35D176A9607053e0c46e91F36AFbC1dc57",
	"to": "0x5592EC0cfb4dbc12D3aB100b257153436a1f0FEa",
	"value": "0",
	"data": "0xa9059cbb00000000000000000000000099d580d3a7fe7bd183b2464517b2cd7ce5a8f15a0000000000000000000000000000000000000000000000000de0b6b3a7640000",
	"operation": 0,
	"gasToken": "0x0000000000000000000000000000000000000000",
	"safeTxGas": 0,
	"baseGas": 0,
	"gasPrice": "0",
	"refundReceiver": "0x0000000000000000000000000000000000000000",
	"nonce": 15,
	"executionDate": "2022-01-10T20:00:12Z",
	"submissionDate": "2022-01-10T19:59:59.689989Z",
	"modified": "2022-01-10T20:00:31.903635Z",
	"blockNumber": 9968802,
	"transactionHash": "0xc9fef30499ee8984974ab9dddd9d15c2a97c1a4393935dceed5efc3af9fc41a4",
	"safeTxHash": "0x6619dab5401503f2735256e12b898e69eb701d6a7e0d07abf1be4bb8aebfba29",
	"executor": "0xbc2BB26a6d821e69A38016f3858561a1D80d4182",
	"isExecuted": true,
	"isSuccessful": true,
	"ethGasPrice": "2500000009",
	"gasUsed": 82902,
	"fee": "207255000746118",
	"chainId": "4",
	"origin": null,
	"dataDecoded": {
		"method": "transfer",
		"parameters": [
				{
				"name": "to",
				"type": "address",
				"value": "0x99D580d3a7FE7BD183b2464517B2cD7ce5A8F15A"
				},
				{
				"name": "value",
				"type": "uint256",
				"value": "1000000000000000000"
				}
		]
	},
	"confirmationsRequired": 1,
	"confirmations": [
		{
		"owner": "0xbc2BB26a6d821e69A38016f3858561a1D80d4182",
		"submissionDate": "2022-01-10T19:59:59.722500Z",
		"transactionHash": null,
		"signature": "0x5ca34641bcdee06e7b99143bfe34778195ca41022bd35837b96c204c7786be9d6dfa6dba43b53cd92da45ac728899e1561b232d28f38ba82df45f164caba38be1b",
		"signatureType": "EOA"
		}
	],
	"signatures": "0x5ca34641bcdee06e7b99143bfe34778195ca41022bd35837b96c204c7786be9d6dfa6dba43b53cd92da45ac728899e1561b232d28f38ba82df45f164caba38be1b"
}
`

func TestGnosisTypedDataWithChainId(t *testing.T) {
	var td apitypes.TypedData
	err := json.Unmarshal([]byte(gnosisTypedDataWithChainId), &td)
	if err != nil {
		t.Fatalf("unmarshalling failed '%v'", err)
	}
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x6619dab5401503f2735256e12b898e69eb701d6a7e0d07abf1be4bb8aebfba29")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}

// TestGnosisCustomData tests the scenario where a user submits only the gnosis-safe
// specific data, and we fill the TypedData struct on our side
func TestGnosisCustomDataWithChainId(t *testing.T) {
	var tx core.GnosisSafeTx
	err := json.Unmarshal([]byte(gnosisTxWithChainId), &tx)
	if err != nil {
		t.Fatal(err)
	}
	var td = tx.ToTypedData()
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x6619dab5401503f2735256e12b898e69eb701d6a7e0d07abf1be4bb8aebfba29")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}

var complexTypedData = `
{
    "types": {
        "EIP712Domain": [
            {
                "name": "chainId",
                "type": "uint256"
            },
            {
                "name": "name",
                "type": "string"
            },
            {
                "name": "verifyingContract",
                "type": "address"
            },
            {
                "name": "version",
                "type": "string"
            }
        ],
        "Action": [
            {
                "name": "action",
                "type": "string"
            },
            {
                "name": "params",
                "type": "string"
            }
        ],
        "Cell": [
            {
                "name": "capacity",
                "type": "string"
            },
            {
                "name": "lock",
                "type": "string"
            },
            {
                "name": "type",
                "type": "string"
            },
            {
                "name": "data",
                "type": "string"
            },
            {
                "name": "extraData",
                "type": "string"
            }
        ],
        "Transaction": [
            {
                "name": "DAS_MESSAGE",
                "type": "string"
            },
            {
                "name": "inputsCapacity",
                "type": "string"
            },
            {
                "name": "outputsCapacity",
                "type": "string"
            },
            {
                "name": "fee",
                "type": "string"
            },
            {
                "name": "action",
                "type": "Action"
            },
            {
                "name": "inputs",
                "type": "Cell[]"
            },
            {
                "name": "outputs",
                "type": "Cell[]"
            },
            {
                "name": "digest",
                "type": "bytes32"
            }
        ]
    },
    "primaryType": "Transaction",
    "domain": {
        "chainId": "56",
        "name": "da.systems",
        "verifyingContract": "0x0000000000000000000000000000000020210722",
        "version": "1"
    },
    "message": {
        "DAS_MESSAGE": "SELL mobcion.bit FOR 100000 CKB",
        "inputsCapacity": "1216.9999 CKB",
        "outputsCapacity": "1216.9998 CKB",
        "fee": "0.0001 CKB",
        "digest": "0x53a6c0f19ec281604607f5d6817e442082ad1882bef0df64d84d3810dae561eb",
        "action": {
            "action": "start_account_sale",
            "params": "0x00"
        },
        "inputs": [
            {
                "capacity": "218 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-cell-type,0x01,0x",
                "data": "{ account: mobcion.bit, expired_at: 1670913958 }",
                "extraData": "{ status: 0, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
            }
        ],
        "outputs": [
            {
                "capacity": "218 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-cell-type,0x01,0x",
                "data": "{ account: mobcion.bit, expired_at: 1670913958 }",
                "extraData": "{ status: 1, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
            },
            {
                "capacity": "201 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-sale-cell-type,0x01,0x",
                "data": "0x1209460ef3cb5f1c68ed2c43a3e020eec2d9de6e...",
                "extraData": ""
            }
        ]
    }
}
`

func TestComplexTypedData(t *testing.T) {
	var td apitypes.TypedData
	err := json.Unmarshal([]byte(complexTypedData), &td)
	if err != nil {
		t.Fatalf("unmarshalling failed '%v'", err)
	}
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x42b1aca82bb6900ff75e90a136de550a58f1a220a071704088eabd5e6ce20446")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}

func TestGnosisSafe(t *testing.T) {
	// json missing chain id
	js := "{\n  \"safe\": \"0x899FcB1437DE65DC6315f5a69C017dd3F2837557\",\n  \"to\": \"0x899FcB1437DE65DC6315f5a69C017dd3F2837557\",\n  \"value\": \"0\",\n  \"data\": \"0x0d582f13000000000000000000000000d3ed2b8756b942c98c851722f3bd507a17b4745f0000000000000000000000000000000000000000000000000000000000000005\",\n  \"operation\": 0,\n  \"gasToken\": \"0x0000000000000000000000000000000000000000\",\n  \"safeTxGas\": 0,\n  \"baseGas\": 0,\n  \"gasPrice\": \"0\",\n  \"refundReceiver\": \"0x0000000000000000000000000000000000000000\",\n  \"nonce\": 0,\n  \"executionDate\": null,\n  \"submissionDate\": \"2022-02-23T14:09:00.018475Z\",\n  \"modified\": \"2022-12-01T15:52:21.214357Z\",\n  \"blockNumber\": null,\n  \"transactionHash\": null,\n  \"safeTxHash\": \"0x6f0f5cffee69087c9d2471e477a63cab2ae171cf433e754315d558d8836274f4\",\n  \"executor\": null,\n  \"isExecuted\": false,\n  \"isSuccessful\": null,\n  \"ethGasPrice\": null,\n  \"maxFeePerGas\": null,\n  \"maxPriorityFeePerGas\": null,\n  \"gasUsed\": null,\n  \"fee\": null,\n  \"origin\": \"https://gnosis-safe.io\",\n  \"dataDecoded\": {\n    \"method\": \"addOwnerWithThreshold\",\n    \"parameters\": [\n      {\n        \"name\": \"owner\",\n        \"type\": \"address\",\n        \"value\": \"0xD3Ed2b8756b942c98c851722F3bd507a17B4745F\"\n      },\n      {\n        \"name\": \"_threshold\",\n        \"type\": \"uint256\",\n        \"value\": \"5\"\n      }\n    ]\n  },\n  \"confirmationsRequired\": 4,\n  \"confirmations\": [\n    {\n      \"owner\": \"0x30B714E065B879F5c042A75Bb40a220A0BE27966\",\n      \"submissionDate\": \"2022-03-01T14:56:22Z\",\n      \"transactionHash\": \"0x6d0a9c83ac7578ef3be1f2afce089fb83b619583dfa779b82f4422fd64ff3ee9\",\n      \"signature\": \"0x00000000000000000000000030b714e065b879f5c042a75bb40a220a0be27966000000000000000000000000000000000000000000000000000000000000000001\",\n      \"signatureType\": \"APPROVED_HASH\"\n    },\n    {\n      \"owner\": \"0x8300dFEa25Da0eb744fC0D98c23283F86AB8c10C\",\n      \"submissionDate\": \"2022-12-01T15:52:21.214357Z\",\n      \"transactionHash\": null,\n      \"signature\": \"0xbce73de4cc6ee208e933a93c794dcb8ba1810f9848d1eec416b7be4dae9854c07dbf1720e60bbd310d2159197a380c941cfdb55b3ce58f9dd69efd395d7bef881b\",\n      \"signatureType\": \"EOA\"\n    }\n  ],\n  \"trusted\": true,\n  \"signatures\": null\n}\n"
	var gnosisTx core.GnosisSafeTx
	if err := json.Unmarshal([]byte(js), &gnosisTx); err != nil {
		t.Fatal(err)
	}
	sighash, _, err := apitypes.TypedDataAndHash(gnosisTx.ToTypedData())
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Equal(sighash, gnosisTx.InputExpHash.Bytes()) {
		t.Fatal("expected inequality")
	}
	gnosisTx.ChainId = (*math.HexOrDecimal256)(big.NewInt(1))
	sighash, _, _ = apitypes.TypedDataAndHash(gnosisTx.ToTypedData())
	if !bytes.Equal(sighash, gnosisTx.InputExpHash.Bytes()) {
		t.Fatal("expected equality")
	}
}

var complexTypedDataLCRefType = `
{
    "types": {
        "EIP712Domain": [
            {
                "name": "chainId",
                "type": "uint256"
            },
            {
                "name": "name",
                "type": "string"
            },
            {
                "name": "verifyingContract",
                "type": "address"
            },
            {
                "name": "version",
                "type": "string"
            }
        ],
        "Action": [
            {
                "name": "action",
                "type": "string"
            },
            {
                "name": "params",
                "type": "string"
            }
        ],
        "cCell": [
            {
                "name": "capacity",
                "type": "string"
            },
            {
                "name": "lock",
                "type": "string"
            },
            {
                "name": "type",
                "type": "string"
            },
            {
                "name": "data",
                "type": "string"
            },
            {
                "name": "extraData",
                "type": "string"
            }
        ],
        "Transaction": [
            {
                "name": "DAS_MESSAGE",
                "type": "string"
            },
            {
                "name": "inputsCapacity",
                "type": "string"
            },
            {
                "name": "outputsCapacity",
                "type": "string"
            },
            {
                "name": "fee",
                "type": "string"
            },
            {
                "name": "action",
                "type": "Action"
            },
            {
                "name": "inputs",
                "type": "cCell[]"
            },
            {
                "name": "outputs",
                "type": "cCell[]"
            },
            {
                "name": "digest",
                "type": "bytes32"
            }
        ]
    },
    "primaryType": "Transaction",
    "domain": {
        "chainId": "56",
        "name": "da.systems",
        "verifyingContract": "0x0000000000000000000000000000000020210722",
        "version": "1"
    },
    "message": {
        "DAS_MESSAGE": "SELL mobcion.bit FOR 100000 CKB",
        "inputsCapacity": "1216.9999 CKB",
        "outputsCapacity": "1216.9998 CKB",
        "fee": "0.0001 CKB",
        "digest": "0x53a6c0f19ec281604607f5d6817e442082ad1882bef0df64d84d3810dae561eb",
        "action": {
            "action": "start_account_sale",
            "params": "0x00"
        },
        "inputs": [
            {
                "capacity": "218 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-cell-type,0x01,0x",
                "data": "{ account: mobcion.bit, expired_at: 1670913958 }",
                "extraData": "{ status: 0, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
            }
        ],
        "outputs": [
            {
                "capacity": "218 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-cell-type,0x01,0x",
                "data": "{ account: mobcion.bit, expired_at: 1670913958 }",
                "extraData": "{ status: 1, records_hash: 0x55478d76900611eb079b22088081124ed6c8bae21a05dd1a0d197efcc7c114ce }"
            },
            {
                "capacity": "201 CKB",
                "lock": "das-lock,0x01,0x051c152f77f8efa9c7c6d181cc97ee67c165c506...",
                "type": "account-sale-cell-type,0x01,0x",
                "data": "0x1209460ef3cb5f1c68ed2c43a3e020eec2d9de6e...",
                "extraData": ""
            }
        ]
    }
}
`

func TestComplexTypedDataWithLowercaseReftype(t *testing.T) {
	var td apitypes.TypedData
	err := json.Unmarshal([]byte(complexTypedDataLCRefType), &td)
	if err != nil {
		t.Fatalf("unmarshalling failed '%v'", err)
	}
	_, sighash, err := sign(td)
	if err != nil {
		t.Fatal(err)
	}
	expSigHash := common.FromHex("0x49191f910874f0148597204d9076af128d4694a7c4b714f1ccff330b87207bff")
	if !bytes.Equal(expSigHash, sighash) {
		t.Fatalf("Error, got %x, wanted %x", sighash, expSigHash)
	}
}
