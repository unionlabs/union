/*******************************************************************************
*   (c) 2018 - 2022 Zondax AG
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
	"fmt"
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_PrintVersion(t *testing.T) {
	reqVersion := VersionInfo{0, 1, 2, 3}
	s := fmt.Sprintf("%v", reqVersion)
	assert.Equal(t, "1.2.3", s)
}

func Test_PathGeneration0(t *testing.T) {
	bip32Path := []uint32{44, 100, 0, 0, 0}

	pathBytes, err := GetBip32bytesv1(bip32Path, 0)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		41,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 41)

	assert.Equal(
		t,
		"052c000000640000000000000000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}

func Test_PathGeneration2(t *testing.T) {
	bip32Path := []uint32{44, 118, 0, 0, 0}

	pathBytes, err := GetBip32bytesv1(bip32Path, 2)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		41,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 41)

	assert.Equal(
		t,
		"052c000080760000800000000000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}

func Test_PathGeneration3(t *testing.T) {
	bip32Path := []uint32{44, 118, 0, 0, 0}

	pathBytes, err := GetBip32bytesv1(bip32Path, 3)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		41,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 41)

	assert.Equal(
		t,
		"052c000080760000800000008000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}

func Test_PathGeneration0v2(t *testing.T) {
	bip32Path := []uint32{44, 100, 0, 0, 0}

	pathBytes, err := GetBip32bytesv2(bip32Path, 0)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		40,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 40)

	assert.Equal(
		t,
		"2c000000640000000000000000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}

func Test_PathGeneration2v2(t *testing.T) {
	bip32Path := []uint32{44, 118, 0, 0, 0}

	pathBytes, err := GetBip32bytesv2(bip32Path, 2)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		40,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 40)

	assert.Equal(
		t,
		"2c000080760000800000000000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}

func Test_PathGeneration3v2(t *testing.T) {
	bip32Path := []uint32{44, 118, 0, 0, 0}

	pathBytes, err := GetBip32bytesv2(bip32Path, 3)

	if err != nil {
		t.Fatalf("Detected error, err: %s\n", err.Error())
	}

	fmt.Printf("Path: %x\n", pathBytes)

	assert.Equal(
		t,
		40,
		len(pathBytes),
		"PathBytes has wrong length: %x, expected length: %x\n", pathBytes, 40)

	assert.Equal(
		t,
		"2c000080760000800000008000000000000000000000000000000000000000000000000000000000",
		fmt.Sprintf("%x", pathBytes),
		"Unexpected PathBytes\n")
}
