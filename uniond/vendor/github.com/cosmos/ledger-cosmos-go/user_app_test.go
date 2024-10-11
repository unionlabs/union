/*******************************************************************************
*   (c) 2018 - 2022 ZondaX AG
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

package ledger_cosmos_go

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/ecdsa"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Ledger Test Mnemonic: equip will roof matter pink blind book anxiety banner elbow sun young

func Test_UserFindLedger(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}

	assert.NotNil(t, userApp)
	defer userApp.Close()
}

func Test_UserGetVersion(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	version, err := userApp.GetVersion()
	require.Nil(t, err, "Detected error")
	fmt.Println(version)

	assert.Equal(t, uint8(0x0), version.AppMode, "TESTING MODE ENABLED!!")
	assert.Equal(t, uint8(0x2), version.Major, "Wrong Major version")
	assert.Equal(t, uint8(0x1), version.Minor, "Wrong Minor version")
	assert.Equal(t, uint8(0x0), version.Patch, "Wrong Patch version")
}

func Test_UserGetPublicKey(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	path := []uint32{44, 118, 5, 0, 21}

	pubKey, err := userApp.GetPublicKeySECP256K1(path)
	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	assert.Equal(t, 33, len(pubKey),
		"Public key has wrong length: %x, expected length: %x\n", pubKey, 65)
	fmt.Printf("PUBLIC KEY: %x\n", pubKey)

	assert.Equal(t,
		"03cb5a33c61595206294140c45efa8a817533e31aa05ea18343033a0732a677005",
		hex.EncodeToString(pubKey),
		"Unexpected pubkey")
}

func Test_GetAddressPubKeySECP256K1_Zero(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	hrp := "cosmos"
	path := []uint32{44, 118, 0, 0, 0}

	pubKey, addr, err := userApp.GetAddressPubKeySECP256K1(path, hrp)
	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("PUBLIC KEY : %x\n", pubKey)
	fmt.Printf("BECH32 ADDR: %s\n", addr)

	assert.Equal(t, 33, len(pubKey), "Public key has wrong length: %x, expected length: %x\n", pubKey, 65)

	assert.Equal(t, "034fef9cd7c4c63588d3b03feb5281b9d232cba34d6f3d71aee59211ffbfe1fe87", hex.EncodeToString(pubKey), "Unexpected pubkey")
	assert.Equal(t, "cosmos1w34k53py5v5xyluazqpq65agyajavep2rflq6h", addr, "Unexpected addr")
}

func Test_GetAddressPubKeySECP256K1(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	hrp := "cosmos"
	path := []uint32{44, 118, 5, 0, 21}

	pubKey, addr, err := userApp.GetAddressPubKeySECP256K1(path, hrp)
	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("PUBLIC KEY : %x\n", pubKey)
	fmt.Printf("BECH32 ADDR: %s\n", addr)

	assert.Equal(t, 33, len(pubKey), "Public key has wrong length: %x, expected length: %x\n", pubKey, 65)

	assert.Equal(t, "03cb5a33c61595206294140c45efa8a817533e31aa05ea18343033a0732a677005", hex.EncodeToString(pubKey), "Unexpected pubkey")
	assert.Equal(t, "cosmos162zm3k8mc685592d7vej2lxrp58mgmkcec76d6", addr, "Unexpected addr")
}

func Test_UserPK_HDPaths(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	path := []uint32{44, 118, 0, 0, 0}

	expected := []string{
		"034fef9cd7c4c63588d3b03feb5281b9d232cba34d6f3d71aee59211ffbfe1fe87",
		"0260d0487a3dfce9228eee2d0d83a40f6131f551526c8e52066fe7fe1e4a509666",
		"03a2670393d02b162d0ed06a08041e80d86be36c0564335254df7462447eb69ab3",
		"033222fc61795077791665544a90740e8ead638a391a3b8f9261f4a226b396c042",
		"03f577473348d7b01e7af2f245e36b98d181bc935ec8b552cde5932b646dc7be04",
		"0222b1a5486be0a2d5f3c5866be46e05d1bde8cda5ea1c4c77a9bc48d2fa2753bc",
		"0377a1c826d3a03ca4ee94fc4dea6bccb2bac5f2ac0419a128c29f8e88f1ff295a",
		"031b75c84453935ab76f8c8d0b6566c3fcc101cc5c59d7000bfc9101961e9308d9",
		"038905a42433b1d677cc8afd36861430b9a8529171b0616f733659f131c3f80221",
		"038be7f348902d8c20bc88d32294f4f3b819284548122229decd1adf1a7eb0848b",
	}

	for i := uint32(0); i < 10; i++ {
		path[4] = i

		pubKey, err := userApp.GetPublicKeySECP256K1(path)
		if err != nil {
			t.Fatalf("Detected error, err: %s\n", err.Error())
		}

		assert.Equal(
			t,
			33,
			len(pubKey),
			"Public key has wrong length: %x, expected length: %x\n", pubKey, 65)

		assert.Equal(
			t,
			expected[i],
			hex.EncodeToString(pubKey),
			"Public key 44'/118'/0'/0/%d does not match\n", i)

		_, err = btcec.ParsePubKey(pubKey[:])
		require.Nil(t, err, "Error parsing public key err: %s\n", err)

	}
}

func getDummyTx() []byte {
	dummyTx := `{
		"account_number": 1,
		"chain_id": "some_chain",
		"fee": {
			"amount": [{"amount": 10, "denom": "DEN"}],
			"gas": 5
		},
		"memo": "MEMO",
		"msgs": ["SOMETHING"],
		"sequence": 3
	}`
	dummyTx = strings.Replace(dummyTx, " ", "", -1)
	dummyTx = strings.Replace(dummyTx, "\n", "", -1)
	dummyTx = strings.Replace(dummyTx, "\t", "", -1)

	return []byte(dummyTx)
}

func Test_UserSign(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	path := []uint32{44, 118, 0, 0, 5}

	message := getDummyTx()
	signature, err := userApp.SignSECP256K1(path, message, 0)
	if err != nil {
		t.Fatalf("[Sign] Error: %s\n", err.Error())
	}

	// Verify Signature
	pubKey, err := userApp.GetPublicKeySECP256K1(path)
	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	if err != nil {
		t.Fatalf("[GetPK] Error: " + err.Error())
		return
	}

	pub2, err := btcec.ParsePubKey(pubKey[:])
	if err != nil {
		t.Fatalf("[ParsePK] Error: " + err.Error())
		return
	}

	sig2, err := ecdsa.ParseDERSignature(signature[:])
	if err != nil {
		t.Fatalf("[ParseSig] Error: " + err.Error())
		return
	}

	hash := sha256.Sum256(message)
	verified := sig2.Verify(hash[:], pub2)
	if !verified {
		t.Fatalf("[VerifySig] Error verifying signature: " + err.Error())
		return
	}
}

func Test_UserSign_Fails(t *testing.T) {
	userApp, err := FindLedgerCosmosUserApp()
	if err != nil {
		t.Fatalf(err.Error())
	}
	defer userApp.Close()

	path := []uint32{44, 118, 0, 0, 5}

	message := getDummyTx()
	garbage := []byte{65}
	message = append(garbage, message...)

	_, err = userApp.SignSECP256K1(path, message, 0)
	assert.Error(t, err)
	errMessage := err.Error()

	if errMessage != "Invalid character in JSON string" && errMessage != "Unexpected characters" {
		assert.Fail(t, "Unexpected error message returned: "+errMessage)
	}
}
