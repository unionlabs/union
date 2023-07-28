package test

import (
	"crypto/rand"
	"math/big"
	mrand "math/rand"
	"reflect"
	"time"

	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
)

var seedCorpus []*big.Int

func init() {
	seedCorpus = make([]*big.Int, 0, 300)

	// small values, including bits
	for i := -5; i <= 5; i++ {
		seedCorpus = append(seedCorpus, big.NewInt(int64(i)))
	}

	// moduli
	for _, curve := range gnark.Curves() {
		fp := curve.BaseField()
		fr := curve.ScalarField()
		seedCorpus = append(seedCorpus, fp)
		seedCorpus = append(seedCorpus, fr)

		var bi big.Int
		for i := -3; i <= 3; i++ {
			bi.SetInt64(int64(i))
			var fp1, fr1 big.Int
			fp1.Add(fp, &bi)
			fr1.Add(fr, &bi)

			seedCorpus = append(seedCorpus, &fp1)
			seedCorpus = append(seedCorpus, &fr1)
		}
	}

	// powers of 2
	bi := big.NewInt(1)
	bi.Lsh(bi, 32)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 64)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 254)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 255)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 256)
	seedCorpus = append(seedCorpus, bi)

}

type filler func(frontend.Circuit, ecc.ID)

func zeroFiller(w frontend.Circuit, curve ecc.ID) {
	fill(w, func() interface{} {
		return 0
	})
}

func binaryFiller(w frontend.Circuit, curve ecc.ID) {
	mrand.Seed(time.Now().Unix())

	fill(w, func() interface{} {
		return int(mrand.Uint32() % 2) //#nosec G404 weak rng is fine here
	})
}

func seedFiller(w frontend.Circuit, curve ecc.ID) {

	mrand.Seed(time.Now().Unix())

	m := curve.ScalarField()

	fill(w, func() interface{} {
		i := int(mrand.Uint32() % uint32(len(seedCorpus))) //#nosec G404 weak rng is fine here
		r := new(big.Int).Set(seedCorpus[i])
		return r.Mod(r, m)
	})
}

func randomFiller(w frontend.Circuit, curve ecc.ID) {

	mrand.Seed(time.Now().Unix())

	r := mrand.New(mrand.NewSource(time.Now().Unix())) //#nosec G404 weak rng is fine here
	m := curve.ScalarField()

	fill(w, func() interface{} {
		i := int(mrand.Uint32() % uint32(len(seedCorpus)*2)) //#nosec G404 weak rng is fine here
		if i >= len(seedCorpus) {
			b1, _ := rand.Int(r, m) //#nosec G404 weak rng is fine here
			return b1
		}
		r := new(big.Int).Set(seedCorpus[i])
		return r.Mod(r, m)
	})
}

func fill(w frontend.Circuit, nextValue func() interface{}) {
	setHandler := func(f schema.LeafInfo, tInput reflect.Value) error {
		v := nextValue()
		tInput.Set(reflect.ValueOf((v)))
		return nil
	}
	// this can't error.
	// TODO @gbotrel it might error with .Walk?
	_, _ = schema.Walk(w, tVariable, setHandler)
}

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
}
