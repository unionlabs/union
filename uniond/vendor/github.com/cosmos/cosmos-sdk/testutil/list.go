package testutil

import (
	"math/rand"
)

func RandSliceElem[E any](r *rand.Rand, elems []E) (E, bool) {
	if len(elems) == 0 {
		var e E
		return e, false
	}

	return elems[r.Intn(len(elems))], true
}
