package fptower

import (
	"testing"

	"github.com/leanovate/gopter"
	"github.com/leanovate/gopter/prop"
)

func TestE2AssemblyOps(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE2()
	genB := GenE2()

	properties.Property("[BN254] mulAsm & mulGeneric should output same result", prop.ForAll(
		func(a, b *E2) bool {
			var c, d E2
			c.Mul(a, b)
			mulGenericE2(&d, a, b)
			return c.Equal(&d)
		},
		genA,
		genB,
	))

	properties.Property("[BN254] squareAsm & squareGeneric should output same result", prop.ForAll(
		func(a *E2) bool {
			var c, d E2
			c.Square(a)
			squareGenericE2(&d, a)
			return c.Equal(&d)
		},
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}
