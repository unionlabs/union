// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package generator

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	field "github.com/consensys/gnark-crypto/field/generator/config"
)

// integration test will create modulus for various field sizes and run tests

const rootDir = "integration_test"

func TestIntegration(t *testing.T) {
	os.RemoveAll(rootDir)
	err := os.MkdirAll(rootDir, 0700)
	defer os.RemoveAll(rootDir)
	if err != nil {
		t.Fatal(err)
	}

	var bits []int
	for i := 64; i <= 448; i += 64 {
		bits = append(bits, i-3, i-2, i-1, i, i+1)
	}

	moduli := make(map[string]string)
	for _, i := range bits {
		var q *big.Int
		var nbWords int
		if i%64 == 0 {
			q, _ = rand.Prime(rand.Reader, i)
			moduli[fmt.Sprintf("e_cios_%04d", i)] = q.String()
		} else {
			for {
				q, _ = rand.Prime(rand.Reader, i)
				nbWords = len(q.Bits())
				const B = (^uint64(0) >> 1) - 1
				if uint64(q.Bits()[nbWords-1]) <= B {
					break
				}
			}
			moduli[fmt.Sprintf("e_nocarry_%04d", i)] = q.String()
		}
	}

	moduli["forty_seven"] = "47"
	moduli["small"] = "9459143039767"
	moduli["small_without_no_carry"] = "18446744073709551557" // 64bits

	moduli["e_secp256k1"] = "115792089237316195423570985008687907853269984665640564039457584007908834671663"

	// JUST fails to be nocarry -- only the following two can occur for < 3000 bits
	moduli["e_nocarry_edge_0127"] = "170141183460469231731687303715884105727"
	moduli["e_nocarry_edge_1279"] = "10407932194664399081925240327364085538615262247266704805319112350403608059673360298012239441732324184842421613954281007791383566248323464908139906605677320762924129509389220345773183349661583550472959420547689811211693677147548478866962501384438260291732348885311160828538416585028255604666224831890918801847068222203140521026698435488732958028878050869736186900714720710555703168729087"

	for elementName, modulus := range moduli {
		var fIntegration *field.FieldConfig
		// generate field
		childDir := filepath.Join(rootDir, elementName)
		fIntegration, err = field.NewFieldConfig("integration", elementName, modulus, false)
		if err != nil {
			t.Fatal(elementName, err)
		}
		if err = GenerateFF(fIntegration, childDir); err != nil {
			t.Fatal(elementName, err)
		}
	}

	// run go test
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	packageDir := filepath.Join(wd, rootDir) + string(filepath.Separator) + "..."
	cmd := exec.Command("go", "test", packageDir)
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			t.Fatal(string(exitErr.Stderr))
		} else {
			t.Fatal(err)
		}
	}

}
