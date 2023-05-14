package fields_bn254

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/std/math/emulated"
)

func init() {
	solver.RegisterHint(GetHints()...)
}

// GetHints returns all hint functions used in the package.
func GetHints() []solver.Hint {
	return []solver.Hint{
		// E2
		divE2Hint,
		inverseE2Hint,
		// E6
		divE6Hint,
		inverseE6Hint,
		squareTorusHint,
		// E12
		divE12Hint,
		inverseE12Hint,
	}
}

func inverseE2Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bn254.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])

			c.Inverse(&a)

			c.A0.BigInt(outputs[0])
			c.A1.BigInt(outputs[1])

			return nil
		})
}

func divE2Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, b, c bn254.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])
			b.A0.SetBigInt(inputs[2])
			b.A1.SetBigInt(inputs[3])

			c.Inverse(&b).Mul(&c, &a)

			c.A0.BigInt(outputs[0])
			c.A1.BigInt(outputs[1])

			return nil
		})
}

// E6 hints
func inverseE6Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bn254.E6

			a.B0.A0.SetBigInt(inputs[0])
			a.B0.A1.SetBigInt(inputs[1])
			a.B1.A0.SetBigInt(inputs[2])
			a.B1.A1.SetBigInt(inputs[3])
			a.B2.A0.SetBigInt(inputs[4])
			a.B2.A1.SetBigInt(inputs[5])

			c.Inverse(&a)

			c.B0.A0.BigInt(outputs[0])
			c.B0.A1.BigInt(outputs[1])
			c.B1.A0.BigInt(outputs[2])
			c.B1.A1.BigInt(outputs[3])
			c.B2.A0.BigInt(outputs[4])
			c.B2.A1.BigInt(outputs[5])

			return nil
		})
}

func divE6Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, b, c bn254.E6

			a.B0.A0.SetBigInt(inputs[0])
			a.B0.A1.SetBigInt(inputs[1])
			a.B1.A0.SetBigInt(inputs[2])
			a.B1.A1.SetBigInt(inputs[3])
			a.B2.A0.SetBigInt(inputs[4])
			a.B2.A1.SetBigInt(inputs[5])

			b.B0.A0.SetBigInt(inputs[6])
			b.B0.A1.SetBigInt(inputs[7])
			b.B1.A0.SetBigInt(inputs[8])
			b.B1.A1.SetBigInt(inputs[9])
			b.B2.A0.SetBigInt(inputs[10])
			b.B2.A1.SetBigInt(inputs[11])

			c.Inverse(&b).Mul(&c, &a)

			c.B0.A0.BigInt(outputs[0])
			c.B0.A1.BigInt(outputs[1])
			c.B1.A0.BigInt(outputs[2])
			c.B1.A1.BigInt(outputs[3])
			c.B2.A0.BigInt(outputs[4])
			c.B2.A1.BigInt(outputs[5])

			return nil
		})
}

func squareTorusHint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bn254.E6

			a.B0.A0.SetBigInt(inputs[0])
			a.B0.A1.SetBigInt(inputs[1])
			a.B1.A0.SetBigInt(inputs[2])
			a.B1.A1.SetBigInt(inputs[3])
			a.B2.A0.SetBigInt(inputs[4])
			a.B2.A1.SetBigInt(inputs[5])

			_c := a.DecompressTorus()
			_c.CyclotomicSquare(&_c)
			c, _ = _c.CompressTorus()

			c.B0.A0.BigInt(outputs[0])
			c.B0.A1.BigInt(outputs[1])
			c.B1.A0.BigInt(outputs[2])
			c.B1.A1.BigInt(outputs[3])
			c.B2.A0.BigInt(outputs[4])
			c.B2.A1.BigInt(outputs[5])

			return nil
		})
}

// E12 hints
func inverseE12Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bn254.E12

			a.C0.B0.A0.SetBigInt(inputs[0])
			a.C0.B0.A1.SetBigInt(inputs[1])
			a.C0.B1.A0.SetBigInt(inputs[2])
			a.C0.B1.A1.SetBigInt(inputs[3])
			a.C0.B2.A0.SetBigInt(inputs[4])
			a.C0.B2.A1.SetBigInt(inputs[5])
			a.C1.B0.A0.SetBigInt(inputs[6])
			a.C1.B0.A1.SetBigInt(inputs[7])
			a.C1.B1.A0.SetBigInt(inputs[8])
			a.C1.B1.A1.SetBigInt(inputs[9])
			a.C1.B2.A0.SetBigInt(inputs[10])
			a.C1.B2.A1.SetBigInt(inputs[11])

			c.Inverse(&a)

			c.C0.B0.A0.BigInt(outputs[0])
			c.C0.B0.A1.BigInt(outputs[1])
			c.C0.B1.A0.BigInt(outputs[2])
			c.C0.B1.A1.BigInt(outputs[3])
			c.C0.B2.A0.BigInt(outputs[4])
			c.C0.B2.A1.BigInt(outputs[5])
			c.C1.B0.A0.BigInt(outputs[6])
			c.C1.B0.A1.BigInt(outputs[7])
			c.C1.B1.A0.BigInt(outputs[8])
			c.C1.B1.A1.BigInt(outputs[9])
			c.C1.B2.A0.BigInt(outputs[10])
			c.C1.B2.A1.BigInt(outputs[11])

			return nil
		})
}

func divE12Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, b, c bn254.E12

			a.C0.B0.A0.SetBigInt(inputs[0])
			a.C0.B0.A1.SetBigInt(inputs[1])
			a.C0.B1.A0.SetBigInt(inputs[2])
			a.C0.B1.A1.SetBigInt(inputs[3])
			a.C0.B2.A0.SetBigInt(inputs[4])
			a.C0.B2.A1.SetBigInt(inputs[5])
			a.C1.B0.A0.SetBigInt(inputs[6])
			a.C1.B0.A1.SetBigInt(inputs[7])
			a.C1.B1.A0.SetBigInt(inputs[8])
			a.C1.B1.A1.SetBigInt(inputs[9])
			a.C1.B2.A0.SetBigInt(inputs[10])
			a.C1.B2.A1.SetBigInt(inputs[11])

			b.C0.B0.A0.SetBigInt(inputs[12])
			b.C0.B0.A1.SetBigInt(inputs[13])
			b.C0.B1.A0.SetBigInt(inputs[14])
			b.C0.B1.A1.SetBigInt(inputs[15])
			b.C0.B2.A0.SetBigInt(inputs[16])
			b.C0.B2.A1.SetBigInt(inputs[17])
			b.C1.B0.A0.SetBigInt(inputs[18])
			b.C1.B0.A1.SetBigInt(inputs[19])
			b.C1.B1.A0.SetBigInt(inputs[20])
			b.C1.B1.A1.SetBigInt(inputs[21])
			b.C1.B2.A0.SetBigInt(inputs[22])
			b.C1.B2.A1.SetBigInt(inputs[23])

			c.Inverse(&b).Mul(&c, &a)

			c.C0.B0.A0.BigInt(outputs[0])
			c.C0.B0.A1.BigInt(outputs[1])
			c.C0.B1.A0.BigInt(outputs[2])
			c.C0.B1.A1.BigInt(outputs[3])
			c.C0.B2.A0.BigInt(outputs[4])
			c.C0.B2.A1.BigInt(outputs[5])
			c.C1.B0.A0.BigInt(outputs[6])
			c.C1.B0.A1.BigInt(outputs[7])
			c.C1.B1.A0.BigInt(outputs[8])
			c.C1.B1.A1.BigInt(outputs[9])
			c.C1.B2.A0.BigInt(outputs[10])
			c.C1.B2.A1.BigInt(outputs[11])

			return nil
		})
}
