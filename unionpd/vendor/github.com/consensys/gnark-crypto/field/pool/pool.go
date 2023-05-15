package pool

import (
	"math/big"
	"sync"
)

// BigInt is a shared *big.Int memory pool
var BigInt bigIntPool

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
	if v == nil {
		return // see https://github.com/ConsenSys/gnark-crypto/issues/316
	}
	_bigIntPool.Put(v)
}
