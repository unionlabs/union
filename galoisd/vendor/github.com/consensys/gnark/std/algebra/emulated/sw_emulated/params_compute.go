package sw_emulated

import (
	"crypto/elliptic"
	"math/big"

	bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761"
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

func computeBLS12381Table() [][2]*big.Int {
	Gjac, _, _, _ := bls12381.Generators()
	table := make([][2]*big.Int, 256)
	tmp := new(bls12381.G1Jac).Set(&Gjac)
	aff := new(bls12381.G1Affine)
	jac := new(bls12381.G1Jac)
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

func computeP256Table() [][2]*big.Int {
	table := make([][2]*big.Int, 256)
	p256 := elliptic.P256()
	gx, gy := p256.Params().Gx, p256.Params().Gy
	tmpx, tmpy := new(big.Int).Set(gx), new(big.Int).Set(gy)
	for i := 1; i < 256; i++ {
		tmpx, tmpy = p256.Double(tmpx, tmpy)
		switch i {
		case 1, 2:
			xx, yy := p256.Add(tmpx, tmpy, gx, gy)
			table[i-1] = [2]*big.Int{xx, yy}
		case 3:
			xx, yy := p256.Add(tmpx, tmpy, gx, new(big.Int).Sub(p256.Params().P, gy))
			table[i-1] = [2]*big.Int{xx, yy}
			fallthrough
		default:
			table[i] = [2]*big.Int{tmpx, tmpy}
		}
	}
	return table
}

func computeP384Table() [][2]*big.Int {
	table := make([][2]*big.Int, 384)
	p384 := elliptic.P384()
	gx, gy := p384.Params().Gx, p384.Params().Gy
	tmpx, tmpy := new(big.Int).Set(gx), new(big.Int).Set(gy)
	for i := 1; i < 384; i++ {
		tmpx, tmpy = p384.Double(tmpx, tmpy)
		switch i {
		case 1, 2:
			xx, yy := p384.Add(tmpx, tmpy, gx, gy)
			table[i-1] = [2]*big.Int{xx, yy}
		case 3:
			xx, yy := p384.Add(tmpx, tmpy, gx, new(big.Int).Sub(p384.Params().P, gy))
			table[i-1] = [2]*big.Int{xx, yy}
			fallthrough
		default:
			table[i] = [2]*big.Int{tmpx, tmpy}
		}
	}
	return table
}

func computeBW6761Table() [][2]*big.Int {
	Gjac, _, _, _ := bw6761.Generators()
	table := make([][2]*big.Int, 377)
	tmp := new(bw6761.G1Jac).Set(&Gjac)
	aff := new(bw6761.G1Affine)
	jac := new(bw6761.G1Jac)
	for i := 1; i < 377; i++ {
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
