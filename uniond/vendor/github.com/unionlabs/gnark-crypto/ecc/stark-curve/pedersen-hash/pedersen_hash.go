package pedersenhash

import (
	"math/big"

	starkcurve "github.com/consensys/gnark-crypto/ecc/stark-curve"
	"github.com/consensys/gnark-crypto/ecc/stark-curve/fp"
)

const nibbleCount = fp.Bits / 4

var (
	shiftPoint   starkcurve.G1Jac
	pointIndexed [4][nibbleCount][16]*starkcurve.G1Jac
	p            [4]starkcurve.G1Jac
)

func init() {
	// The curve points come from the [reference implementation].
	//
	// [reference implementation]: https://github.com/starkware-libs/cairo-lang/blob/de741b92657f245a50caab99cfaef093152fd8be/src/starkware/crypto/signature/fast_pedersen_hash.py

	shiftPoint.X.SetString("2089986280348253421170679821480865132823066470938446095505822317253594081284")
	shiftPoint.Y.SetString("1713931329540660377023406109199410414810705867260802078187082345529207694986")
	shiftPoint.Z.SetOne()

	p[0].X.SetString("996781205833008774514500082376783249102396023663454813447423147977397232763")
	p[0].Y.SetString("1668503676786377725805489344771023921079126552019160156920634619255970485781")
	p[0].Z.SetOne()

	p[1].X.SetString("2251563274489750535117886426533222435294046428347329203627021249169616184184")
	p[1].Y.SetString("1798716007562728905295480679789526322175868328062420237419143593021674992973")
	p[1].Z.SetOne()

	p[2].X.SetString("2138414695194151160943305727036575959195309218611738193261179310511854807447")
	p[2].Y.SetString("113410276730064486255102093846540133784865286929052426931474106396135072156")
	p[2].Z.SetOne()

	p[3].X.SetString("2379962749567351885752724891227938183011949129833673362440656643086021394946")
	p[3].Y.SetString("776496453633298175483985398648758586525933812536653089401905292063708816422")
	p[3].Z.SetOne()

	var multiplier big.Int
	for pointIndex, point := range p {
		var nibbleIndexed [nibbleCount][16]*starkcurve.G1Jac
		for nibIndex := uint(0); nibIndex < nibbleCount; nibIndex++ {
			var selectorIndexed [16]*starkcurve.G1Jac
			for selector := 0; selector < 16; selector++ {
				multiplier.SetUint64(uint64(selector))
				multiplier.Lsh(&multiplier, nibIndex*4)

				res := point
				res.ScalarMultiplication(&res, &multiplier)
				selectorIndexed[selector] = &res
			}
			nibbleIndexed[nibIndex] = selectorIndexed
		}
		pointIndexed[pointIndex] = nibbleIndexed
	}
}

// PedersenArray implements [Pedersen array hashing].
//
// [Pedersen array hashing]: https://docs.starknet.io/documentation/develop/Hashing/hash-functions/#array_hashing
func PedersenArray(elems ...*fp.Element) fp.Element {
	var d fp.Element
	for _, e := range elems {
		d = Pedersen(&d, e)
	}
	return Pedersen(&d, new(fp.Element).SetUint64(uint64(len(elems))))
}

// Pedersen implements the [Pedersen hash] based on the [reference implementation].
//
// [Pedersen hash]: https://docs.starknet.io/documentation/develop/Hashing/hash-functions/#pedersen_hash
// [reference implementation]: https://github.com/starkware-libs/cairo-lang/blob/de741b92657f245a50caab99cfaef093152fd8be/src/starkware/crypto/signature/fast_pedersen_hash.py
func Pedersen(a *fp.Element, b *fp.Element) fp.Element {
	acc := shiftPoint
	accumulate := func(bytes []byte, nibbleIndexed [nibbleCount][16]*starkcurve.G1Jac) {
		for i, val := range bytes {
			lowNibble := val & 0x0F
			index := len(bytes) - i - 1

			if lowNibble > 0 {
				lowNibbleIndex := 2 * index
				acc.AddAssign(nibbleIndexed[lowNibbleIndex][lowNibble])
			}

			highNibble := (val & 0xF0) >> 4

			if highNibble > 0 {
				highNibbleIndex := (2 * index) + 1
				acc.AddAssign(nibbleIndexed[highNibbleIndex][highNibble])
			}
		}
	}

	aBytes := a.Bytes()
	accumulate(aBytes[1:], pointIndexed[0])
	accumulate(aBytes[:1], pointIndexed[1])
	bBytes := b.Bytes()
	accumulate(bBytes[1:], pointIndexed[2])
	accumulate(bBytes[:1], pointIndexed[3])

	// recover the affine x coordinate
	var x fp.Element
	x.Inverse(&acc.Z).
		Square(&x)
	x.Mul(&acc.X, &x)

	return x
}
