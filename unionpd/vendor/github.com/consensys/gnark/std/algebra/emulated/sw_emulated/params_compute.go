package sw_emulated

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/secp256k1"
)

func computeSecp256k1Table() [][2]*big.Int {
	Gjac, _ := secp256k1.Generators()
	table := make([][2]*big.Int, 256)
	tmp := new(secp256k1.G1Jac).Set(&Gjac)
	aff := new(secp256k1.G1Affine)
	jac := new(secp256k1.G1Jac)
	for i := 1; i < 256; i++ {
		tmp = tmp.Double(tmp)
		switch i {
		case 1, 2:
			jac.Set(tmp).AddAssign(&Gjac)
			aff.FromJacobian(jac)
			table[i-1] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
		case 3:
			jac.Set(tmp).SubAssign(&Gjac)
			aff.FromJacobian(jac)
			table[i-1] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
			fallthrough
		default:
			aff.FromJacobian(tmp)
			table[i] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
		}
	}
	return table[:]
}

func computeBN254Table() [][2]*big.Int {
	Gjac, _, _, _ := bn254.Generators()
	table := make([][2]*big.Int, 256)
	tmp := new(bn254.G1Jac).Set(&Gjac)
	aff := new(bn254.G1Affine)
	jac := new(bn254.G1Jac)
	for i := 1; i < 256; i++ {
		tmp = tmp.Double(tmp)
		switch i {
		case 1, 2:
			jac.Set(tmp).AddAssign(&Gjac)
			aff.FromJacobian(jac)
			table[i-1] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
		case 3:
			jac.Set(tmp).SubAssign(&Gjac)
			aff.FromJacobian(jac)
			table[i-1] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
			fallthrough
		default:
			aff.FromJacobian(tmp)
			table[i] = [2]*big.Int{aff.X.BigInt(new(big.Int)), aff.Y.BigInt(new(big.Int))}
		}
	}
	return table
}
