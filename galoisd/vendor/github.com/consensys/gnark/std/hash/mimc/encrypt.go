/*
Copyright © 2020 ConsenSys

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package mimc

import (
	"github.com/consensys/gnark-crypto/ecc"
	bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr/mimc"
	bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr/mimc"
	bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr/mimc"
	bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr/mimc"
	bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr/mimc"
	bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr/mimc"

	"github.com/consensys/gnark/frontend"
)

var encryptFuncs map[ecc.ID]func(MiMC, frontend.Variable) frontend.Variable
var newMimc map[ecc.ID]func(frontend.API) MiMC

func init() {
	encryptFuncs = make(map[ecc.ID]func(MiMC, frontend.Variable) frontend.Variable)
	encryptFuncs[ecc.BN254] = encryptPow5
	encryptFuncs[ecc.BLS12_381] = encryptPow5
	encryptFuncs[ecc.BLS12_377] = encryptPow17
	encryptFuncs[ecc.BW6_761] = encryptPow5
	encryptFuncs[ecc.BW6_633] = encryptPow5
	encryptFuncs[ecc.BLS24_315] = encryptPow5
	encryptFuncs[ecc.BLS24_317] = encryptPow7

	newMimc = make(map[ecc.ID]func(frontend.API) MiMC)
	newMimc[ecc.BN254] = newMimcBN254
	newMimc[ecc.BLS12_381] = newMimcBLS381
	newMimc[ecc.BLS12_377] = newMimcBLS377
	newMimc[ecc.BW6_761] = newMimcBW761
	newMimc[ecc.BW6_633] = newMimcBW633
	newMimc[ecc.BLS24_315] = newMimcBLS315
	newMimc[ecc.BLS24_317] = newMimcBLS317
}

// -------------------------------------------------------------------------------------------------
// constructors

func newMimcBLS377(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bls12377.GetConstants()
	res.id = ecc.BLS12_377
	res.h = 0
	res.api = api
	return res
}

func newMimcBLS381(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bls12381.GetConstants()
	res.id = ecc.BLS12_381
	res.h = 0
	res.api = api
	return res
}

func newMimcBN254(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bn254.GetConstants()
	res.id = ecc.BN254
	res.h = 0
	res.api = api
	return res
}

func newMimcBW761(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bw6761.GetConstants()
	res.id = ecc.BW6_761
	res.h = 0
	res.api = api
	return res
}

func newMimcBLS317(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bls24317.GetConstants()
	res.id = ecc.BLS24_317
	res.h = 0
	res.api = api
	return res
}

func newMimcBLS315(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bls24315.GetConstants()
	res.id = ecc.BLS24_315
	res.h = 0
	res.api = api
	return res
}

func newMimcBW633(api frontend.API) MiMC {
	res := MiMC{}
	res.params = bw6633.GetConstants()
	res.id = ecc.BW6_633
	res.h = 0
	res.api = api
	return res
}

// -------------------------------------------------------------------------------------------------
// encryptions functions

func pow5(api frontend.API, x frontend.Variable) frontend.Variable {
	r := api.Mul(x, x)
	r = api.Mul(r, r)
	return api.Mul(r, x)
}

func pow7(api frontend.API, x frontend.Variable) frontend.Variable {
	t := api.Mul(x, x)
	r := api.Mul(t, t)
	r = api.Mul(r, t)
	return api.Mul(r, x)
}

func pow17(api frontend.API, x frontend.Variable) frontend.Variable {
	r := api.Mul(x, x)
	r = api.Mul(r, r)
	r = api.Mul(r, r)
	r = api.Mul(r, r)
	return api.Mul(r, x)
}

// encryptBn256 of a mimc run expressed as r1cs
// m is the message, k the key
func encryptPow5(h MiMC, m frontend.Variable) frontend.Variable {
	x := m
	for i := 0; i < len(h.params); i++ {
		x = pow5(h.api, h.api.Add(x, h.h, h.params[i]))
	}
	return h.api.Add(x, h.h)
}

// encryptBLS24317 of a mimc run expressed as r1cs
// m is the message, k the key
func encryptPow7(h MiMC, m frontend.Variable) frontend.Variable {
	x := m
	for i := 0; i < len(h.params); i++ {
		x = pow7(h.api, h.api.Add(x, h.h, h.params[i]))
	}
	return h.api.Add(x, h.h)
}

// encryptBLS377 of a mimc run expressed as r1cs
// m is the message, k the key
func encryptPow17(h MiMC, m frontend.Variable) frontend.Variable {
	x := m
	for i := 0; i < len(h.params); i++ {
		// res = (res+key+c)**17
		x = pow17(h.api, h.api.Add(x, h.h, h.params[i]))
	}
	return h.api.Add(x, h.h)

}
