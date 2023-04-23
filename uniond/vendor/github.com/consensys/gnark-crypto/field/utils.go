package field

import (
	"fmt"
	"math/big"
	"math/bits"
	"sync"
)

// BigIntPool is a shared *big.Int memory pool
var BigIntPool bigIntPool

var _bigIntPool = sync.Pool{
	New: func() interface{} {
		return new(big.Int)
	},
}

type bigIntPool struct{}

func (bigIntPool) Get() *big.Int {
	return _bigIntPool.Get().(*big.Int)
}

func (bigIntPool) Put(v *big.Int) {
	_bigIntPool.Put(v)
}

// BigIntMatchUint64Slice is a test helper to match big.Int words againt a uint64 slice
func BigIntMatchUint64Slice(aInt *big.Int, a []uint64) error {

	words := aInt.Bits()

	const steps = 64 / bits.UintSize
	const filter uint64 = 0xFFFFFFFFFFFFFFFF >> (64 - bits.UintSize)
	for i := 0; i < len(a)*steps; i++ {

		var wI big.Word

		if i < len(words) {
			wI = words[i]
		}

		aI := a[i/steps] >> ((i * bits.UintSize) % 64)
		aI &= filter

		if uint64(wI) != aI {
			return fmt.Errorf("bignum mismatch: disagreement on word %d: %x ≠ %x; %d ≠ %d", i, uint64(wI), aI, uint64(wI), aI)
		}
	}

	return nil
}
